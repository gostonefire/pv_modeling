mod errors;
mod logging;
mod initialization;
mod handlers;
mod manager_production;
mod manager_fox_cloud;
mod models;
mod manager_weather;
mod cache;
mod serialize_timestamp;

use actix_web::{middleware, web, App, HttpServer};
use actix_files::Files;
use log::info;
use crate::errors::UnrecoverableError;
use crate::handlers::{get_data, get_start};
use crate::initialization::{config, Config};

struct AppState {
    config: Config,
}

#[actix_web::main]
async fn main() -> Result<(), UnrecoverableError> {
    let config = config()?;
    let web_data = web::Data::new(AppState { config: config.clone() });

    info!("starting web server");
    HttpServer::new(move || {
        App::new()
            .app_data(web_data.clone())
            .service(get_data)
            .service(get_start)
            .service(
                web::scope("")
                    .wrap(middleware::DefaultHeaders::new().add(("Cache-Control", "no-cache")))
                    .service(Files::new("/", "./static").index_file("index.html"))
            )
    })
        .bind((config.web_server.bind_address.as_str(), config.web_server.bind_port))?
        .disable_signals()
        .run()
        .await?;

    Ok(())
}
