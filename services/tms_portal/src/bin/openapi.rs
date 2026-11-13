use tms_portal::presentation::http::v1::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    println!(
        "{}",
        ApiDoc::openapi()
            .to_json()
            .expect("OpenAPI document serializes")
    );
}
