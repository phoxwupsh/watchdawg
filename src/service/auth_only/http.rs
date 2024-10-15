use super::AuthOnlySvc;
use crate::service::{TcpService, TcpServiceError};
use async_trait::async_trait;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

pub struct HttpAuthOnly {
    service: AuthOnlySvc,
}

impl HttpAuthOnly {
    pub fn new(service: AuthOnlySvc) -> Self {
        Self { service }
    }
}

#[async_trait]
impl TcpService for HttpAuthOnly {
    async fn serve_tcp(&self, incoming: TcpStream) -> Result<(), TcpServiceError> {
        let io = TokioIo::new(incoming);
        let auth_svc = self.service.clone();
        hyper::server::conn::http1::Builder::new()
            .serve_connection(io, auth_svc)
            .await
            .map_err(TcpServiceError::from)
    }
}
