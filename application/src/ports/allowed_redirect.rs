use crate::obj_model::allowed_redirect::AllowedRedirect;

#[async_trait::async_trait]
pub trait AllowedRedirectRepository: Send + Sync {
    async fn get_by_client_id_and_uri(
        &self,
        client_id: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<AllowedRedirect>;

    async fn get_by_client_name_and_uri(
        &self,
        client_name: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<AllowedRedirect>;
}
