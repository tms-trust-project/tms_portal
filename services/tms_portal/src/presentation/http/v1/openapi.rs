use utoipa::OpenApi;

use super::delegations::handlers::{
    add_delegation::__path_add_delegation,
    list_delegations::__path_list_delegations,
    list_delegations_by_client_id::__path_list_delegations_by_client_id,
    delete_delegation::__path_delete_delegation,
};

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:3000", description = "Local TMS Portal HTTP server"),
    ),
    paths(
        add_delegation,
        list_delegations,
        list_delegations_by_client_id,
        delete_delegation,
    ),
    components(schemas()),
    info(title = "TMS Portal", version="1.0.0")
)]
pub struct ApiDoc;
