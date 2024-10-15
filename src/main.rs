use std::sync::Arc;
use argh::FromArgs;
use auth::htpasswd::HtpasswdAuth;
use client::{http::HttpClient, https::HttpsClient, ProxyClient};
use config::Config;
use server::ProxyServer;
use service::{
    auth_only::{http::HttpAuthOnly, https::HttpsAuthOnly, AuthOnlySvc},
    auth_reverse_proxy::{http::HttpAuthRevPrx, https::HttpsAuthRevPrx, AuthRevPrxSvc},
};
use session::{memory::MemoryStore, redis::RedisStore, SessionManager, SessionStore};
use thiserror::Error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use utils::{load_certs, load_private_key};

mod auth;
mod client;
mod config;
mod server;
mod service;
mod session;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = argh::from_env::<Args>();
    let config = Config::from_file(args.config.unwrap_or("config.toml".into()))?;

    let log_level = match config.debug {
        true => LevelFilter::DEBUG,
        false => LevelFilter::INFO,
    };

    tracing_subscriber::fmt::fmt()
        .with_max_level(log_level)
        .finish()
        .init();

    let addr = (config.listen_address.clone(), config.listen_port);
    let session_store: Arc<dyn SessionStore + Send + Sync> = match config.session.storage.as_str() {
        "memory" => Arc::new(MemoryStore::new()),
        "redis" => Arc::new(RedisStore::new(
            &config
                .session
                .redis_conn
                .ok_or(ServerError::MissingProperty("session.redis_conn"))?,
        )?),
        _ => {
            return Err(
                std::io::Error::other("Session storage should be `memory` or `redis`").into(),
            )
        }
    };

    let session_manager = SessionManager::new(
        config.session.cookie_name.as_str(),
        session_store,
        config.session.session_expire,
    );
    let authenticator = HtpasswdAuth::new("htpasswd")?;

    let server = match config.reverse_proxy.enabled {
        false => {
            let service = AuthOnlySvc::new(
                authenticator,
                &config
                    .auth_return_header_name
                    .ok_or(ServerError::MissingProperty("auth_return_header_name"))?,
                session_manager,
            )?;
            match config.https.enabled {
                true => {
                    let certs = load_certs(
                        &config
                            .https
                            .cert
                            .ok_or(ServerError::MissingProperty("https.cert"))?,
                    )?;
                    let key = load_private_key(
                        &config
                            .https
                            .key
                            .ok_or(ServerError::MissingProperty("https.key"))?,
                    )?;
                    ProxyServer::new(HttpsAuthOnly::new(certs, key, service)?)
                }
                false => ProxyServer::new(HttpAuthOnly::new(service)),
            }
        }
        true => {
            let proxy_client: Arc<dyn ProxyClient + Send + Sync> = match config.https.enabled {
                true => Arc::new(HttpsClient::new()?),
                false => Arc::new(HttpClient),
            };
            let service = AuthRevPrxSvc::new(
                config
                    .reverse_proxy
                    .proxy_address
                    .ok_or(ServerError::MissingProperty("reverse_proxy.proxy_address"))?
                    .as_str(),
                authenticator,
                session_manager,
                proxy_client,
            )?;

            match config.https.enabled {
                true => {
                    let certs = load_certs(
                        &config
                            .https
                            .cert
                            .ok_or(ServerError::MissingProperty("https.cert"))?,
                    )?;
                    let key = load_private_key(
                        &config
                            .https
                            .key
                            .ok_or(ServerError::MissingProperty("https.cert"))?,
                    )?;
                    ProxyServer::new(HttpsAuthRevPrx::new(certs, key, service)?)
                }
                false => ProxyServer::new(HttpAuthRevPrx::new(service)),
            }
        }
    };

    // server.run_workers(addr, config.workers)?;
    server.run(addr)?;
    Ok(())
}

#[derive(FromArgs)]
#[argh(
    description = "An authentication server for nginx's \"auth_request\", using HTTP basic authentication and htpasswd, can also work standalone"
)]
struct Args {
    #[argh(
        option,
        description = "the path to the config file, \"config.toml\" by default"
    )]
    config: Option<String>,
}

#[derive(Error, Debug)]
enum ServerError {
    #[error("Require property `{0}` to be set in config file")]
    MissingProperty(&'static str),
}
