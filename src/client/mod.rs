use async_trait::async_trait;
use http_body_util::combinators::BoxBody;
use hyper::{
    body::{Bytes, Incoming},
    Request, Response, Version,
};
use rustls_pki_types::ServerName;
use std::net::SocketAddr;
use thiserror::Error;

pub mod http;
pub mod https;

#[async_trait]
pub trait ProxyClient {
    async fn proxy_request(
        &self,
        addr: SocketAddr,
        domain: ServerName<'static>,
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, ProxyClientError>;
    fn default_port(&self) -> u16;
}

#[derive(Error, Debug)]
pub enum ProxyClientError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // #[error(transparent)]
    // Tls(#[from] native_tls::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),

    #[error("HTTP version `{0:?}` not supported")]
    NotSupportHttpVersion(Version),
}
