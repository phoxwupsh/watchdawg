use std::sync::Arc;

use super::AuthOnlySvc;
use crate::service::{TcpService, TcpServiceError};
use async_trait::async_trait;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

pub struct HttpsAuthOnly {
    service: AuthOnlySvc,
    tls_acceptor: TlsAcceptor,
}

impl HttpsAuthOnly {
    pub fn new(
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
        service: AuthOnlySvc,
    ) -> Result<Self, rustls::Error> {
        let mut server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        server_config.alpn_protocols.push(b"h2".to_vec());
        server_config.alpn_protocols.push(b"http/1.1".to_vec());
        server_config.alpn_protocols.push(b"http/1.0".to_vec());

        let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

        Ok(Self {
            tls_acceptor,
            service,
        })
    }
}

#[async_trait]
impl TcpService for HttpsAuthOnly {
    async fn serve_tcp(&self, incoming: TcpStream) -> Result<(), TcpServiceError> {
        let tls_stream = match self.tls_acceptor.accept(incoming).await {
            Ok(stream) => stream,
            Err(err) => return Err(TcpServiceError::Io(err)),
        };
        let io = TokioIo::new(tls_stream);

        let service = self.service.clone();

        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(TcpServiceError::from)
    }
}
