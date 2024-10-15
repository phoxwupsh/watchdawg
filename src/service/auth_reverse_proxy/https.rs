use super::AuthRevPrxSvc;
use crate::service::TcpServiceError;
use async_trait::async_trait;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

pub struct HttpsAuthRevPrx {
    service: crate::AuthRevPrxSvc,
    tls_acceptor: TlsAcceptor,
}

impl HttpsAuthRevPrx {
    pub fn new(
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
        service: AuthRevPrxSvc,
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
impl crate::service::TcpService for HttpsAuthRevPrx {
    async fn serve_tcp(
        &self,
        incoming: tokio::net::TcpStream,
    ) -> Result<(), crate::service::TcpServiceError> {
        let tls_stream = match self.tls_acceptor.accept(incoming).await {
            Ok(stream) => stream,
            Err(err) => return Err(TcpServiceError::Io(err)),
        };
        let io = TokioIo::new(tls_stream);
        let service = self.service.clone();

        hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
            .serve_connection(io, service)
            .await?;

        Ok(())
    }
}
