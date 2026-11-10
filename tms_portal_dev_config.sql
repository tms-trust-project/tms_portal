-- Add the globus identity provider (as login identity provider)
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
                'openid profile email', 'globus', true, false);

-- Add the tacc identity provider (as resource provider)
INSERT INTO identity_providers (id, name, oauth2_client_id, oauth2_client_secret, identity_redirect_url, 
oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, scope, provider_type, supports_login, 
supports_resources, resources_services_url) 
VALUES ('tacc', 'TACC Resource Provider', '${TACC_RP_CLIENT_ID}',
'${TACC_RP_CLIENT_SECRET}',
'https://tacc.tapis.io/v3/oauth2/authorize', 'https://tacc.tapis.io/v3/oauth2/tokens', 
'https://tacc.tapis.io/v3/tokens/.well-known/jwks.json', '',
'openid profile email', 'tacc_tapis', false, true, '${RESOURCES_ENDPOINT}');

-- Add tms client
INSERT INTO clients (client_id, name, secret, enabled) 
        VALUES('tms', 'Tms Service', 'tms', true);
-- Add allowed redirects (with and without trailing slash)
INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('tms', '${BASE_URL}');

INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('tms', '${BASE_URL}/');

-- Add allowed redirects for resource provider oauth callback
INSERT INTO allowed_redirects (uri, client_id)
        VALUES ('${BASE_URL}/resources/providers/callback', 'tms');

-- Add tapisauth client for tms-test tenant
INSERT INTO clients (client_id, name, secret, enabled) 
        VALUES('${TAPIS_AUTH_CLIENT_ID}', '${TAPIS_TEST_TENANT_CLIENT_ID}', '${TAPIS_TEST_TENANT_CLIENT_SECRET}', true);

-- Add allowed redirects for tapis tms-test tenant oauth callback
INSERT INTO allowed_redirects (uri, client_id)
        VALUES ('https://linktest.develop.tapis.io/v3/oauth2/extensions/oa2/callback', '${TAPIS_AUTH_CLIENT_ID}');

-- Add a local test client, for trying the hosted sign in page as another app would.  Start a
-- login at ${BASE_URL}/oauth2/authorize?response_type=code&client_id=testapp&redirect_uri=http://localhost:9999/callback&state=xyz
-- and exchange the code with this client's id and secret.  Dev only: the secret is public.
INSERT INTO clients (client_id, name, secret, enabled) 
        VALUES('testapp', 'Local Test App', 'testapp-secret', true);

INSERT INTO allowed_redirects (client_id, uri) 
        VALUES('testapp', 'http://localhost:9999/callback');
