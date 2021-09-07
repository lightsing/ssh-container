#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::http::header::ContentType;
use actix_web::middleware::Logger;
use actix_web::{get, guard, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use internal::auth::cas::{CasResponse, ServiceResponse};
use internal::{AuthStatus, AuthStore, ChallengeFilter, Config, DaemonConfig};
use serde::Deserialize;
use tokio::sync::RwLock;

#[actix_web::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let config = Arc::new(DaemonConfig::new("ssh-containerd.conf")?);
    let config_ = config.clone();
    let challenge_filter: ChallengeFilter = Arc::new(RwLock::new(HashMap::new()));
    let auth_status: AuthStore = Arc::new(RwLock::new(HashMap::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(challenge_filter.clone()))
            .app_data(web::Data::new(auth_status.clone()))
            .service(web::scope("/auth").service(cas))
            .service(
                web::scope("/manage")
                    .guard(guard::fn_guard(|head| {
                        head.peer_addr
                            .map(|addr| match addr {
                                SocketAddr::V4(v4) => v4.ip().is_loopback(),
                                SocketAddr::V6(v6) => {
                                    v6.ip().is_loopback()
                                        || v6
                                            .ip()
                                            .to_ipv4()
                                            .map(|v4| v4.is_loopback())
                                            .unwrap_or(false)
                                }
                            })
                            .unwrap_or(false)
                    }))
                    .service(create_auth_url)
                    .service(get_auth_status),
            )
    })
    .bind(config_.server().bind())?
    .run()
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct GetAuthUrlRequest {
    identifier: String,
}

#[post("/auth-url")]
async fn create_auth_url(
    req: web::Json<GetAuthUrlRequest>,
    config: web::Data<Arc<DaemonConfig>>,
    auth_status: web::Data<AuthStore>,
    challenge_filter: web::Data<ChallengeFilter>,
) -> impl Responder {
    let req = req.into_inner();
    let challenge = uuid::Uuid::new_v4().to_string();
    let auth_url = config
        .auth()
        .cas()
        .unwrap()
        .gen_auth_url(challenge.as_str());
    challenge_filter
        .write()
        .await
        .insert(challenge.clone(), req.identifier.clone());
    auth_status
        .write()
        .await
        .insert(req.identifier, AuthStatus::Assigned);
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body(auth_url)
}

#[get("/auth-status/{identifier}")]
async fn get_auth_status(
    identifier: web::Path<String>,
    auth_status: web::Data<AuthStore>,
) -> impl Responder {
    let status = auth_status
        .read()
        .await
        .get(identifier.as_ref())
        .map(ToOwned::to_owned)
        .unwrap_or(AuthStatus::Failed);
    HttpResponse::Ok().json(serde_json::json!({ "status": status }))
}

#[derive(Debug, Clone, Deserialize)]
struct CasCallbackData {
    challenge: String,
    ticket: String,
}

#[get("/cas")]
async fn cas(
    config: web::Data<Arc<DaemonConfig>>,
    challenge_filter: web::Data<ChallengeFilter>,
    auth_status: web::Data<AuthStore>,
    callback_data: web::Query<CasCallbackData>,
) -> impl Responder {
    if config.auth().cas().is_none() {
        return HttpResponse::Forbidden().finish();
    }
    if !challenge_filter
        .read()
        .await
        .contains_key(&callback_data.challenge)
    {
        return HttpResponse::BadRequest().finish();
    }
    let identifier = challenge_filter
        .write()
        .await
        .remove(&callback_data.challenge)
        .unwrap()
        .to_owned();

    let url = config
        .auth()
        .cas()
        .unwrap()
        .gen_request_url(&*callback_data.challenge, &*callback_data.ticket);
    let client = reqwest::Client::new();
    let response: CasResponse = client.get(url).send().await.unwrap().json().await.unwrap();
    debug!("{}", response);
    let response = match response.service_response {
        ServiceResponse::AuthenticationSuccess(success) => {
            let sid = &success.attributes.sid[0];
            let old = auth_status
                .write()
                .await
                .insert(identifier, AuthStatus::Authed(sid.to_owned()));
            debug_assert_eq!(old, Some(AuthStatus::Assigned));
            HttpResponse::Ok().body(format!("Welcome back, {}", sid))
        }
        ServiceResponse::AuthenticationFailure(failure) => {
            auth_status.write().await.remove(&identifier);
            HttpResponse::Forbidden().json(failure)
        }
    };
    response
}
