use async_trait::async_trait;
use thiserror::Error;
use tokio::net::TcpStream;

pub mod auth_only;
pub mod auth_reverse_proxy;

#[async_trait]
pub trait TcpService {
    async fn serve_tcp(&self, incoming: TcpStream) -> Result<(), TcpServiceError>;
}

#[derive(Error, Debug)]
pub enum TcpServiceError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
