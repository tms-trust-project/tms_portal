use serde::Deserialize;

/*
An approved sign-in option, as returned by tms_server's GET /v1/tms/institutions.
Optional properties are omitted by tms_server when they have no value.
 */
#[derive(Debug, Clone, Deserialize)]
pub struct Institution {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub redirect_idp_domain: Option<String>,
    pub idp_logout_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct InstitutionsResponse {
    pub result_code: String,
    pub result_msg: String,
    pub institutions: Vec<Institution>,
}
