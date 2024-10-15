use crate::service::TcpService;
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};
use tracing::error;

pub struct ProxyServer {
    service: Arc<dyn TcpService + Send + Sync + 'static>,
}

impl ProxyServer {
    pub fn new(service: impl TcpService + Send + Sync + 'static) -> Self {
        let service = Arc::new(service);
        Self { service }
    }

    pub fn run(&self, addr: impl ToSocketAddrs) -> Result<(), std::io::Error> {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async move {
            let listener = TcpListener::bind(addr).await?;
            loop {
                let (stream, _addr) = listener.accept().await?;
                let service = self.service.clone();
                tokio::spawn(async move {
                    if let Err(err) = service.serve_tcp(stream).await {
                        error!("Failed to handle connection from: {}", err);
                    }
                });
            }
        })
    }
}

