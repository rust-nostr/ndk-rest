// Copyright (c) 2023 Nostr Development Kit Devs
// Distributed under the MIT software license

use actix_cors::Cors;
use actix_web::{error, web, App, HttpResponse, HttpServer};
use nostr_sdk::{Client, Keys, Options, Result};
use serde_json::json;

mod config;
mod handler;

use self::config::Config;

pub struct AppState {
    client: Client,
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    let config = Config::get();

    let keys = Keys::generate();
    let opts = Options::new().wait_for_send(true);
    let client = Client::new_with_opts(&keys, opts);

    for url in config.nostr.relays.into_iter() {
        client.add_relay(url, None).await?;
    }

    client.connect().await;

    let http_server = HttpServer::new(move || {
        let json_config = web::JsonConfig::default().error_handler(|err, _req| {
            error::InternalError::from_response(
                "",
                HttpResponse::BadRequest().json(json!({
                    "success": false,
                    "code": 400,
                    "message": err.to_string(),
                    "data": {}
                })),
            )
            .into()
        });

        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .max_age(3600);

        let data = web::Data::new(AppState {
            client: client.clone(),
        });

        App::new()
            .wrap(cors)
            .app_data(json_config)
            .app_data(data)
            .configure(init_routes)
    });

    let server = http_server.bind(config.network.listen_addr)?;

    log::info!("REST API listening on {}", config.network.listen_addr);

    Ok(server.run().await?)
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::ping);
    cfg.service(handler::publish_event);
}
