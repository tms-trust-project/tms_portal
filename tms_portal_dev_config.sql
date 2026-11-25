-- Add the globus identity provider (as login identity provider).  view_identity_set puts
-- every linked identity in the id token (identity_set), so a login through an institution
-- is accepted when any of them is from the institution's domain.
INSERT INTO identity_providers
        (id, name, oauth2_client_id, oauth2_client_secret, identity_redirect_url,
                oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                scope, provider_type, supports_login,
                supports_resources)
VALUES
        ('globus', 'Globus IDP', '${GLOBUS_CLIENT_ID}',
                '${GLOBUS_CLIENT_SECRET}',
                'https://auth.globus.org/v2/oauth2/authorize',
                'https://auth.globus.org/v2/oauth2/token',
                'https://auth.globus.org/jwk.json', '',
                'openid profile email urn:globus:auth:scope:auth.globus.org:view_identity_set', 'globus', true, false)
ON CONFLICT DO NOTHING;

-- Add the tacc identity provider (as resource provider)
INSERT INTO identity_providers (id, name, oauth2_client_id, oauth2_client_secret, identity_redirect_url,
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope, provider_type, supports_login,
supports_resources, resources_services_url)
VALUES ('tacc', 'TACC Resource Provider', '${TACC_RP_CLIENT_ID}',
'${TACC_RP_CLIENT_SECRET}',
'https://tacc.tapis.io/v3/oauth2/authorize', 'https://tacc.tapis.io/v3/oauth2/tokens',
'https://tacc.tapis.io/v3/tokens/.well-known/jwks.json', '',
'openid profile email', 'tacc_tapis', false, true, '${RESOURCES_ENDPOINT}')
ON CONFLICT DO NOTHING;

-- Add tms client
INSERT INTO clients (client_id, name, secret, enabled)
        VALUES('tms', 'Tms Service', 'tms', true)
ON CONFLICT DO NOTHING;

-- Add allowed redirects (with and without trailing slash)
INSERT INTO allowed_redirects (client_id, uri)
SELECT 'tms', '${BASE_URL}'
WHERE NOT EXISTS (
        SELECT 1 FROM allowed_redirects
        WHERE client_id = 'tms' AND uri = '${BASE_URL}'
);

INSERT INTO allowed_redirects (client_id, uri)
SELECT 'tms', '${BASE_URL}/'
WHERE NOT EXISTS (
        SELECT 1 FROM allowed_redirects
        WHERE client_id = 'tms' AND uri = '${BASE_URL}/'
);

-- Add allowed redirects for resource provider oauth callback
INSERT INTO allowed_redirects (uri, client_id)
SELECT '${BASE_URL}/resources/providers/callback', 'tms'
WHERE NOT EXISTS (
        SELECT 1 FROM allowed_redirects
        WHERE client_id = 'tms'
          AND uri = '${BASE_URL}/resources/providers/callback'
);

-- Add tapisauth client for tms-test tenant
INSERT INTO clients (client_id, name, secret, enabled)
        VALUES('${TAPIS_AUTH_CLIENT_ID}', '${TAPIS_TEST_TENANT_CLIENT_ID}', '${TAPIS_TEST_TENANT_CLIENT_SECRET}', true)
ON CONFLICT DO NOTHING;

-- Add allowed redirects for tapis tms-test tenant oauth callback
INSERT INTO allowed_redirects (uri, client_id)
SELECT 'https://linktest.develop.tapis.io/v3/oauth2/extensions/oa2/callback', '${TAPIS_AUTH_CLIENT_ID}'
WHERE NOT EXISTS (
        SELECT 1 FROM allowed_redirects
        WHERE client_id = '${TAPIS_AUTH_CLIENT_ID}'
          AND uri = 'https://linktest.develop.tapis.io/v3/oauth2/extensions/oa2/callback'
);

-- jsonb_build_object mpreserves the PEM key line breaks as valid JSON strings.
INSERT INTO configuration (config_name, config_value)
VALUES
        (
                'http_config',
                jsonb_build_object(
                        'base_url', '${BASE_URL}',
                        'identity_provider_callback_endpoint', 'login/callback',
                        'resource_provider_callback_endpoint', 'resources/providers/callback',
                        'oauth_provider_callback_endpoint', 'oauth2/callback',
                        'token_endpoint', 'oauth2/token',
                        'authorization_endpoint', 'oauth2/authorization',
                        'revocation_endpoint', 'oauth2/token/revoke',
                        'jwks_endpoint', 'jwks.json'
                )
        ),
        (
                'oauth_config',
                jsonb_build_object(
                        'login_oauth_provider', '${LOGIN_OAUTH_PROVIDER}'
                )
        ),
        (
                'jwt_config',
                jsonb_build_object(
                        'default_expiration_minutes', ${JWT_EXPIRATION_MINUTES},
                        'signing_key_kid', '${TMS_TOKEN_KID}',
                        'key', jsonb_build_object(
                                'kid', '${TMS_TOKEN_KID}',
                                'jwt_public_key', $tms_token_public_key$${TMS_TOKEN_PUB_KEY}$tms_token_public_key$,
                                'jwt_private_key', $tms_token_private_key$${TMS_TOKEN_PRIV_KEY}$tms_token_private_key$
                        )
                )
        ),
        (
                'runtime_config',
                jsonb_build_object(
                        'config_directory', '${CONFIG_DIR}',
                        'logging_config_file_name', 'log4rs.yml'
                )
        ),
        (
                'delegation_policy_config',
                jsonb_build_object(
                        'delegation_expiration',
                        to_char(
                                (CURRENT_TIMESTAMP + INTERVAL '30 days') AT TIME ZONE 'UTC',
                                'YYYY-MM-DD"T"HH24:MI:SS.US"Z"'
                        ),
                        'delegation_max_mins_since_login', ${DELEGATION_MAX_MINS_SINCE_LOGIN}
                )
        ),
        (
                'state_key',
                jsonb_build_object(
                        'kid', '${TMS_STATE_KID}',
                        'key', jsonb_build_object(
                                'kid', '${TMS_STATE_KID}',
                                'jwt_public_key', $tms_state_public_key$${TMS_STATE_PUB_KEY}$tms_state_public_key$,
                                'jwt_private_key', $tms_state_private_key$${TMS_STATE_PRIV_KEY}$tms_state_private_key$
                        )
                )
        )
ON CONFLICT DO NOTHING;
