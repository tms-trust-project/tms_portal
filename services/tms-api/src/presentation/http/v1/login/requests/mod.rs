use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthorizeRequest {
    pub redirect_uri: String,
    pub client_return_uri: Option<String>,
    pub client_name: Option<String>,
    // the id of an institution from /institutions, to log in through its own identity
    // provider (the web client's institution picker sends it).  Required: a login without
    // one goes back to the sign in page with ?error=institution_required.
    pub institution: Option<String>,
}
