use crate::obj_model;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct WhoAmIResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    pub username: Value,
    #[serde(skip_serializing_if = "Option::is_none", rename = "idpDisplayName")]
    pub idp_display_name: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<Value>,
    // a sign-out page the browser should open, for providers the portal can't redirect through
    #[serde(skip_serializing_if = "Option::is_none", rename = "logoutUrl")]
    pub logout_url: Option<String>,
}

impl From<crate::obj_model::login::WhoAmIResponse> for WhoAmIResponse {
    fn from(value: crate::obj_model::login::WhoAmIResponse) -> Self {
        WhoAmIResponse {
            name: value.name,
            username: value.username,
            idp_display_name: value.idp_display_name,
            organization: value.organization,
            logout_url: value.logout_url,
        }
    }
}

#[derive(Debug, Serialize, Hash, Eq, PartialEq, Clone)]
pub struct IdentityProvider {
    pub id: String,
    pub name: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "oauth2TokenUrl")]
    pub oauth2_token_url: String,
    #[serde(rename = "userInfoUrl")]
    pub user_info_url: Option<String>,
    pub created: String,
    pub updated: String,
}

impl From<obj_model::identity_provider::IdentityProvider> for IdentityProvider {
    fn from(value: obj_model::identity_provider::IdentityProvider) -> Self {
        IdentityProvider {
            id: value.id,
            name: value.name,
            client_id: value.client_id,
            oauth2_token_url: value.oauth2_token_url,
            user_info_url: value.oidc_user_info_url,
            created: value.created.to_rfc3339(),
            updated: value.updated.to_rfc3339(),
        }
    }
}
impl From<&obj_model::identity_provider::IdentityProvider> for IdentityProvider {
    fn from(value: &obj_model::identity_provider::IdentityProvider) -> Self {
        IdentityProvider {
            id: value.id.clone(),
            name: value.name.clone(),
            client_id: value.client_id.clone(),
            oauth2_token_url: value.oauth2_token_url.clone(),
            user_info_url: value.oidc_user_info_url.clone(),
            created: value.created.to_rfc3339().clone(),
            updated: value.updated.to_rfc3339().clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub redirect_uri: String,
    pub client_return_uri: Option<String>,
    pub client_name: Option<String>,
    pub institution: Option<String>,
}

/*
A sign-in option for the login landing page.  The redirect domain is not sent to
the browser; login_handler looks it up again by id.
 */
#[derive(Debug, Serialize)]
pub struct LoginInstitution {
    pub id: String,
    pub name: String,
    pub provider: String,
}

impl From<&obj_model::institution::Institution> for LoginInstitution {
    fn from(value: &obj_model::institution::Institution) -> Self {
        LoginInstitution {
            id: value.id.clone(),
            name: value.name.clone(),
            provider: value.provider.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LogoutIdpRequest {
    pub institution: String,
}
