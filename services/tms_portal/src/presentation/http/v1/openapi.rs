use utoipa::OpenApi;

use super::delegations::handlers::{
    add_delegation::__path_add_delegation, delete_delegation::__path_delete_delegation,
    list_delegations::__path_list_delegations,
    list_delegations_by_client_id::__path_list_delegations_by_client_id,
};

use super::clients::handlers::{
    get_client_by_id::__path_get_client_by_id,
    list_clients::__path_list_clients,
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
        get_client_by_id,
        list_clients,
    ),
    components(schemas()),
    info(title = "TMS Portal", version="1.0.0")
)]
pub struct ApiDoc;
