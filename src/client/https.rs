use crate::client::ProxyClientError;
use async_trait::async_trait;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{Bytes, Incoming},
    Request, Response, Version,
};
use hyper_rustls::ConfigBuilderExt;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls_pki_types::ServerName;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;
use tracing::{debug, error};

const HTTPS_DEFAULT_PORT: u16 = 443;

pub struct HttpsClient {
    tls_connector: tokio_rustls::TlsConnector,
}

impl HttpsClient {
    pub fn new() -> std::io::Result<Self> {
        let mut client_config = rustls::client::ClientConfig::builder()
            .with_native_roots()?
            .with_no_client_auth();

        client_config.alpn_protocols.push(b"h2".to_vec());
        client_config.alpn_protocols.push(b"http/1.1".to_vec());
        client_config.alpn_protocols.push(b"http/1.0".to_vec());

        let tls_connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

        Ok(Self { tls_connector })
    }
}

#[async_trait]
impl crate::client::ProxyClient for HttpsClient {
    async fn proxy_request(
        &self,
        dest: SocketAddr,
        domain: ServerName<'static>,
        req: Request<Incoming>,
    ) -> Result<Response<BoxBody<Bytes, hyper::Error>>, ProxyClientError> {
        debug!("Forwarding request: {:#?}", req);

        let socket = TcpStream::connect(dest).await?;
        let stream = self.tls_connector.connect(domain, socket).await?;
        let io = TokioIo::new(stream);

        let http_version = req.version();
        let (parts, body) = match http_version {
            Version::HTTP_10 | Version::HTTP_11 => {
                let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        error!("Connection failed: {}", err);
                    }
                });

                sender.send_request(req).await?.into_parts()
            }
            Version::HTTP_2 => {
                let (mut sender, conn) =
                    hyper::client::conn::http2::handshake(TokioExecutor::new(), io).await?;
                tokio::task::spawn(async move {
                    if let Err(err) = conn.await {
                        error!("Connection failed: {}", err);
                    }
                });

                sender.send_request(req).await?.into_parts()
            }
            _ => return Err(ProxyClientError::NotSupportHttpVersion(http_version))
        };

        Ok(Response::from_parts(parts, body.boxed()))
    }

    fn default_port(&self) -> u16 {
        HTTPS_DEFAULT_PORT
    }
}

