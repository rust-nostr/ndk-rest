// Copyright (c) 2023 Nostr Development Kit Devs
// Distributed under the MIT software license

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{error, web, App, HttpResponse, HttpServer};
use nostr_sdk::{Client, Keys, Options, Result};
use redis::Client as RedisClient;
use serde_json::json;

mod config;
mod handler;
mod logger;

use self::config::Config;

pub struct AppState {
    config: Config,
    client: Client,
    redis: Option<RedisClient>,
}

#[actix_web::main]
async fn main() -> Result<()> {
    let config = Config::get();

    logger::init(&config);

    let keys = Keys::generate();
    let opts = Options::new().wait_for_send(true);
    let client = Client::new_with_opts(&keys, opts);

    for url in config.nostr.relays.iter() {
        client.add_relay(url.to_string(), None).await?;
    }

    client.connect().await;

    let redis: Option<RedisClient> = if config.redis.enabled {
        Some(RedisClient::open("redis://127.0.0.1/")?)
    } else {
        None
    };

    let data = web::Data::new(AppState {
        config: config.clone(),
        client,
        redis,
    });

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

        let cors = if config.network.permissive_cors {
            Cors::permissive()
        } else {
            Cors::default()
                .allowed_methods(vec!["GET", "POST"])
                .allow_any_origin()
                .max_age(3600)
        };

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(json_config)
            .app_data(data.clone())
            .configure(init_routes)
    });

    let server = http_server.bind(config.network.listen_addr)?;

    log::info!("REST API listening on {}", config.network.listen_addr);

    Ok(server.run().await?)
}

fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::ping);
    cfg.service(handler::publish_event);
    cfg.service(handler::get_events);
    cfg.service(handler::get_profile);
    cfg.service(handler::get_contacts);
}
