use crate::obj_model::configuration::RuntimeConfig;
use log::info;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::path::Path;

pub async fn init_db(connection_url: &str) -> PgPool {
    // unwrap - panic and exist if we can't conenct to db
    PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_url)
        .await
        .unwrap()
}

pub async fn init_logging(runtime_config: &RuntimeConfig) {
    let config_dir = Path::new(runtime_config.config_directory.as_str());
    let logging_config_path = config_dir.join(runtime_config.logging_config_file_name.as_str());
    match log4rs::init_file(logging_config_path, Default::default()) {
        Ok(_) => (),
        Err(error) => {
            println!("{}", error);
            let msg = format!("Error while initializing logging config: {}", error);
            panic!("{}", msg);
        }
    }
    info!("Log4rs initialized");
}
