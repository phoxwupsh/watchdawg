use crate::client::{ProxyClient, ProxyClientError};
use async_trait::async_trait;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{Bytes, Incoming},
    Request, Response,
};
use hyper_util::rt::TokioIo;
use rustls_pki_types::ServerName;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::error;

const HTTP_DEFAULT_PORT: u16 = 80;

pub struct HttpClient;

#[async_trait]
impl ProxyClient for HttpClient {
    async fn proxy_request(
        &self,
        dest: SocketAddr,
        _domain: ServerName<'static>,
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, ProxyClientError> {
        let stream = TcpStream::connect(dest).await?;

        let io = TokioIo::new(stream);
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection failed: {}", err);
            }
        });

        let (parts, body) = sender.send_request(req).await?.into_parts();
        Ok(Response::from_parts(parts, body.boxed()))
    }
    fn default_port(&self) -> u16 {
        HTTP_DEFAULT_PORT
    }
}
