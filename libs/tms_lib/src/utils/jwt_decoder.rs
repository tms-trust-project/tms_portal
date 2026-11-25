use crate::utils::service_error::ServiceError::Internal;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode, decode_header};
use serde::Deserialize;
use std::collections::HashSet;

pub struct JwtDecoderBuilder {
    jwks_url: Option<String>,
    public_key_bytes: Option<Vec<u8>>,
    validate_aud: bool,
    audience: Option<HashSet<String>>,
    issuer: Option<String>,
}

impl JwtDecoderBuilder {
    pub fn builder() -> JwtDecoderBuilder {
        JwtDecoderBuilder {
            jwks_url: None,
            public_key_bytes: None,
            audience: None,
            validate_aud: true,
            issuer: None,
        }
    }
    pub fn jwks_url(mut self, jwks_url: &Option<String>) -> Self {
        if let Some(jwks_url) = jwks_url {
            self.jwks_url = Some(String::from(jwks_url));
        }
        self
    }

    pub fn public_key(mut self, key_bytes: &[u8]) -> Self {
        self.public_key_bytes = Some(key_bytes.to_vec());
        self
    }

    pub fn validate_aud(mut self, validate_aud: bool) -> Self {
        self.validate_aud = validate_aud;
        self
    }

    pub fn audience(mut self, audience: HashSet<String>) -> Self {
        self.audience = Some(audience);
        self
    }

    // The token's iss must be this issuer (and the token must have an iss).
    pub fn issuer(mut self, issuer: &str) -> Self {
        self.issuer = Some(String::from(issuer));
        self
    }

    pub async fn decode<T>(&self, token: &str) -> anyhow::Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut algorithm = Algorithm::RS256;
        // if jwks url is provided, use that.
        let decoding_key = match self.jwks_url {
            Some(ref jwks_url) => {
                let jwks = get_public_key(jwks_url).await?;
                let token_header = decode_header(token)?;
                algorithm = token_header.alg;

                let jwk = match token_header.kid {
                    Some(kid) => jwks.find(&kid),
                    None => jwks.keys.first(),
                };

                match jwk {
                    Some(jwk) => DecodingKey::from_jwk(jwk)?,
                    None => {
                        return Err(Internal("Unable to find the proper key".to_string()).into());
                    }
                }
            }

            None => match &self.public_key_bytes {
                Some(public_key) => DecodingKey::from_rsa_pem(public_key.as_ref())?,
                None => {
                    return Err(Internal("No key available for decoding JWT".to_string()).into());
                }
            },
        };

        // TODO: remember this trick for generics.  It helps a lot!!
        // println!("Type is: {}", std::any::type_name::<T>());
        let mut validation = Validation::new(algorithm);
        // With validate_aud (the default) the token's aud must be in audience, and a token
        // that has an aud is refused when no audience was given.  Decoders of tokens whose
        // aud isn't ours to check turn it off with validate_aud(false).
        validation.validate_aud = self.validate_aud;
        if let Some(audience) = &self.audience {
            validation.aud = Some(audience.clone());
            validation.required_spec_claims.insert(String::from("aud"));
        }
        if let Some(issuer) = &self.issuer {
            validation.set_issuer(&[issuer]);
            validation.required_spec_claims.insert(String::from("iss"));
        }

        let decoded: TokenData<T> = decode(token, &decoding_key, &validation)?;
        Ok(decoded.claims)
    }
}

async fn get_public_key(pub_key_url: &str) -> anyhow::Result<JwkSet> {
    let client = reqwest::Client::new();
    let response = client.get(pub_key_url).send().await?;
    let jwks_string = response.text().await?;
    serde_json::from_str(jwks_string.as_str()).map_err(|error| error.into())
}

#[cfg(test)]
#[path = "jwt_decoder.test.rs"]
mod jwt_decoder_test;
