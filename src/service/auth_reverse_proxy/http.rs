use super::AuthRevPrxSvc;
use crate::service::{TcpService, TcpServiceError};
use async_trait::async_trait;
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;
use tracing::error;

pub struct HttpAuthRevPrx {
    service: AuthRevPrxSvc,
}

impl HttpAuthRevPrx {
    pub fn new(service: AuthRevPrxSvc) -> Self {
        Self { service }
    }
}

#[async_trait]
impl TcpService for HttpAuthRevPrx {
    async fn serve_tcp(&self, incoming: TcpStream) -> Result<(), TcpServiceError> {
        let io = TokioIo::new(incoming);
        let rev_prx = self.service.clone();
        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, rev_prx)
                .await
            {
                error!("Failed to handle connection: {}", err);
            }
        });
        Ok(())
    }
}
