// extern crate core;

use std::sync::Arc;

use anyhow::Context;
use axum::body::Body;
use tms_portal::config::{init_db, init_logging};
use tms_portal::infra::persistence::client_repository::PostgresClientRepository;
use tms_portal::infra::persistence::delegation_repository::PostgresDelegationRepository;
use tms_portal::infra::persistence::issued_token_repository::db_cleanup_tokens;
use tms_portal::infra::persistence::resource_provider_login_repository::PostgresResourceProviderLoginRepository;
use tms_portal::presentation::http::v1::{
    delegations::router as delegations_router, login::router as login_router,
    oauth2::router as oauth_router, openapi::ApiDoc, resources::router as resources_router,
    well_known::router as well_known_router,
};
use tms_portal::services::delegation_service::DelegationService;
use tms_portal::utils::app_error::AppError;
use tms_portal::utils::configuration::Configuration;
use tms_portal::{AppState, database_url};
//use axum_extra::extract::cookie::Key;
use axum::handler::HandlerWithoutStateExt;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{Router, middleware};
use chrono::{TimeDelta, Utc};
use http::Request;
use log::error;
use tms_lib::utils::service_error::ServiceError::{MethodNotAllowed, NotFound};
use tokio::spawn;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

#[tokio::main]
#[instrument]
async fn main() -> anyhow::Result<()> {
    let database_url_string = database_url()?;
    dbg!(&database_url_string);
    let db_pool = init_db(&database_url_string).await;
    let config = Configuration::from_env()?;
    let client_repo = PostgresClientRepository::new(db_pool.clone());
    let delegation_repo = PostgresDelegationRepository::new(db_pool.clone());
    let resource_provider_login_repo =
        PostgresResourceProviderLoginRepository::new(db_pool.clone());
    let delegation_service = DelegationService::new(
        Arc::new(client_repo),
        Arc::new(delegation_repo),
        Arc::new(resource_provider_login_repo),
        config.delegation_policy_config.clone(),
    );
    let state = AppState {
        // // Generate a secure key
        // //
        // // TODO:  You probably don't wanna generate a new one each time the app starts though
        // key: Key::generate(),
        db_pool: db_pool.clone(),
        delegation_service,
        config,
    };

    init_logging(&state.config.runtime_config).await;
    let cleanup_db_pool = state.db_pool.clone();
    spawn(async move {
        // TODO: configuration setting
        let mut interval = tokio::time::interval(tokio::time::Duration::from_mins(15));
        println!("Begin cleanup thread");
        loop {
            interval.tick().await;
            println!("Token Cleanup");

            let result: anyhow::Result<()> = async {
                match cleanup_db_pool.begin().await {
                    Ok(mut tx) => {
                        // expired an hour ago
                        let expires_before = Utc::now() - TimeDelta::hours(1);
                        match db_cleanup_tokens(&mut tx, &expires_before).await {
                            Ok(_) => tx
                                .commit()
                                .await
                                .context("Unable to commit transaction for token cleanup"),
                            Err(error) => Err(error),
                        }
                    }
                    Err(error) => Err(error.into()),
                }
            }
            .await;

            if let Err(result) = result {
                error!("Failed to perform database cleanup: {}", result);
            }
        }
    });

    let port = 8080;

    println!("Server running on port {0}", &port);

    log_mdc::insert("request_id", "HELLO WORLD");

    let delegations_router: Router<AppState> = delegations_router().await.into();
    let openapi_json = ApiDoc::openapi().to_json()?;

    // build our application with a single route
    let app = Router::new()
        //        .merge(auth::router().await)
        .merge(well_known_router().await)
        .merge(login_router().await)
        .merge(resources_router().await)
        .merge(oauth_router().await)
        .merge(delegations_router)
        .layer(middleware::from_fn(|request: Request<Body>, next: Next| async move {
            let request_id = Uuid::new_v4().to_string();
            log_mdc::insert("request_id", request_id.clone());
            next.run(request).await
        }))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .route(
            "/api-docs/openapi.json",
            axum::routing::get(move || {
                let openapi_json = openapi_json.clone();
                async move { ([("content-type", "application/json")], openapi_json) }
            }),
        )
        .nest_service("/debug", ServeFile::new("dist/debug.html"))
        .nest_service("/assets", ServeDir::new("dist/assets"))
        .fallback_service(ServeDir::new("dist/").not_found_service(not_found.into_service()))
        .method_not_allowed_fallback(method_not_allowed);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &port))
        .await
        .unwrap();

    Ok(axum::serve(listener, app).await?)
}

async fn not_found() -> impl IntoResponse {
    let app_error: AppError = NotFound(String::from("Invalid Path")).into();
    app_error.into_response()
}

async fn method_not_allowed() -> impl IntoResponse {
    let app_error: AppError = MethodNotAllowed(String::from("The method is not allowed")).into();
    app_error.into_response()
}
