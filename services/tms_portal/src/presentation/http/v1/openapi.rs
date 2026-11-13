use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::presentation::http::v1::delegations::handlers::add_delegation::add_delegation
    ),
    tags(
        (name = "delegations", description = "Delegated access to resource-provider accounts")
    )
)]
pub struct ApiDoc;
