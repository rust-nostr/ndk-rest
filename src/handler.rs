// Copyright (c) 2023 Nostr Development Kit Devs
// Distributed under the MIT software license

use actix_web::{get, post, web, HttpResponse};
use nostr::prelude::FromBech32;
use nostr_sdk::hashes::sha256::Hash as Sha256Hash;
use nostr_sdk::hashes::Hash;
use nostr_sdk::prelude::FromPkStr;
use nostr_sdk::prelude::XOnlyPublicKey;
use nostr_sdk::{Client, Event, Filter, Keys, Kind, Url};
use redis::AsyncCommands;
use serde_json::json;

use crate::AppState;

#[get("/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "pong",
        "data": {},
    }))
}

#[post("/event")]
pub async fn publish_event(data: web::Data<AppState>, body: web::Json<Event>) -> HttpResponse {
    let event: Event = body.0;

    if let Err(e) = event.verify() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string(),
            "data": {},
        }));
    }

    match data.client.send_event(event).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Event published",
            "data": {},
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string(),
            "data": {},
        })),
    }
}

#[get("/v1/{relay}/{pubkey}/contacts")]
pub async fn get_contacts(
    _data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (relay, pubkey) = path.into_inner();

    let keys = Keys::from_pk_str(pubkey.as_str()).unwrap();
    // let opts = Options::new().wait_for_send(true);
    // let endpoint = Url::parse(&relay).unwrap();
    let client = Client::new(&keys);

    client.add_relay(relay, None).await.unwrap();
    client.connect().await;

    // println!("pubkey: {}", pubkey);
    // let pk: XOnlyPublicKey = XOnlyPublicKey::from_bech32(pubkey).unwrap();

    match client.get_contact_list_metadata(None).await {
        Ok(contacts) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": format!("Got {} events", contacts.len()),
            "data": contacts,
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string(),
            "data": {},
        })),
    }
}

#[get("/profile/{relay}/{pubkey}")]
pub async fn get_profile(
    _data: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (relay, pubkey) = path.into_inner();

    let keys = Keys::generate();
    // let opts = Options::new().wait_for_send(true);
    let client = Client::new(&keys); //, opts);

    client.add_relay(relay, None).await.unwrap();

    client.connect().await;

    println!("pubkey: {}", pubkey);
    let pk: XOnlyPublicKey = XOnlyPublicKey::from_bech32(pubkey).unwrap();

    let filter = Filter::new()
        .authors(vec![pk])
        .kind(Kind::Metadata)
        .limit(1);

    let mut filters: Vec<Filter> = Vec::new();
    filters.push(filter);

    match client.get_events_of(filters, None).await {
        Ok(events) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": format!("Got {} events", events.len()),
            "data": events,
        })),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e.to_string(),
            "data": {},
        })),
    }
}

#[post("/events")]
pub async fn get_events(data: web::Data<AppState>, body: web::Json<Vec<Filter>>) -> HttpResponse {
    let filters: Vec<Filter> = body.0;

    if filters.len() > data.config.limit.max_filters {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": format!("Too many filters (max allowed {})", data.config.limit.max_filters),
            "data": {},
        }));
    }

    if let Some(redis) = &data.redis {
        let mut connection = redis.get_async_connection().await.unwrap();
        let hash: String = Sha256Hash::hash(format!("{filters:?}").as_bytes()).to_string();
        match connection.exists::<&str, bool>(&hash).await {
            Ok(exists) => {
                if exists {
                    match connection.get(&hash).await {
                        Ok(result) => {
                            let bytes: Vec<u8> = result;
                            let events: Vec<Event> = bincode::deserialize(&bytes).unwrap();
                            HttpResponse::Ok().json(json!({
                                "success": true,
                                "message": format!("Got {} events", events.len()),
                                "data": events,
                            }))
                        }
                        Err(e) => HttpResponse::BadRequest().json(json!({
                            "success": false,
                            "message": e.to_string(),
                            "data": {},
                        })),
                    }
                } else {
                    match data.client.get_events_of(filters, None).await {
                        Ok(events) => {
                            let encoded: Vec<u8> = bincode::serialize(&events).unwrap();
                            let _: () = connection
                                .set_ex(hash, encoded, data.config.redis.expiration)
                                .await
                                .unwrap();
                            HttpResponse::Ok().json(json!({
                                "success": true,
                                "message": format!("Got {} events", events.len()),
                                "data": events,
                            }))
                        }
                        Err(e) => HttpResponse::BadRequest().json(json!({
                            "success": false,
                            "message": e.to_string(),
                            "data": {},
                        })),
                    }
                }
            }
            Err(e) => HttpResponse::BadRequest().json(json!({
                "success": false,
                "message": e.to_string(),
                "data": {},
            })),
        }
    } else {
        // TODO: add a timeout
        match data.client.get_events_of(filters, None).await {
            Ok(events) => HttpResponse::Ok().json(json!({
                "success": true,
                "message": format!("Got {} events", events.len()),
                "data": events,
            })),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "success": false,
                "message": e.to_string(),
                "data": {},
            })),
        }
    }
}
