use crate::{
    auth::Authenticator,
    client::ProxyClient,
    session::SessionManager,
    utils::{header_has_valid_auth, headers_has_valid_session, req_auth},
};
use concat_string::concat_string;
use http_body_util::combinators::BoxBody;
use hyper::{
    body::{Bytes, Incoming},
    header::{HeaderValue, AUTHORIZATION, HOST, SET_COOKIE},
    service::Service,
    Request, Response, Uri,
};
use rustls_pki_types::ServerName;
use std::{
    future::Future,
    net::{SocketAddr, ToSocketAddrs},
    pin::Pin,
    sync::Arc,
};
use tracing::{debug, error};

pub mod http;
pub mod https;

#[derive(Clone)]
pub struct AuthRevPrxSvc {
    inner: Arc<AuthRevPrxSvcImpl>,
}

impl AuthRevPrxSvc {
    pub fn new(
        dest: impl Into<String>,
        authenticator: impl Authenticator + Send + Sync + 'static,
        session_manager: SessionManager,
        client: Arc<dyn ProxyClient + Send + Sync>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let dest: String = dest.into();
        let dest = dest.parse::<Uri>()?;
        let port = dest
            .port()
            .map(|port| port.as_u16())
            .unwrap_or(client.default_port());
        let host = dest.host().unwrap().to_owned();
        let addr = (host.as_str(), port).to_socket_addrs()?.next().unwrap();

        let host_header = HeaderValue::from_str(host.as_str())?;

        debug!("Proxy to domain: {}, address: {}", host, addr);

        let max_age_str = session_manager.max_age.to_string();
        let cookie_name_str = session_manager.cookie_name.clone();
        let cookie_generator = Box::new(move |session_id: &str| {
            concat_string!(
                cookie_name_str,
                "=",
                session_id,
                "; HttpOnly; Max-Age=",
                max_age_str
            )
        });

        let inner: AuthRevPrxSvcImpl = AuthRevPrxSvcImpl {
            auth: Arc::new(authenticator),
            session_manager,
            domain: ServerName::try_from(host)?,
            addr,
            client,
            host_header,
            cookie_generator,
        };
        Ok(AuthRevPrxSvc {
            inner: inner.into(),
        })
    }
}

struct AuthRevPrxSvcImpl {
    auth: Arc<dyn Authenticator + Send + Sync + 'static>,
    session_manager: SessionManager,
    domain: ServerName<'static>,
    addr: SocketAddr,
    client: Arc<dyn ProxyClient + Send + Sync>,
    host_header: HeaderValue,
    cookie_generator: Box<dyn Fn(&str) -> String + Send + Sync>,
}

impl Service<Request<Incoming>> for AuthRevPrxSvc {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
        debug!("Receive request: {:?}", req);

        let mut set_cookie = None;
        let headers = req.headers_mut();

        if headers_has_valid_session(headers, &self.inner.session_manager).is_none() {
            if !header_has_valid_auth(headers, self.inner.auth.as_ref()) {
                return Box::pin(async move { Ok(req_auth()) });
            } else {
                let new_session = self.inner.session_manager.create_session();
                let cookie = (self.inner.cookie_generator)(&new_session);
                set_cookie = Some(cookie);
            }
        }

        if headers
            .get(AUTHORIZATION)
            .map(|header| header.as_bytes())
            .map(|header| header.starts_with(b"Basic "))
            .unwrap_or(false)
        {
            headers.remove(AUTHORIZATION);
        }

        if let Some(host) = headers.get_mut(HOST) {
            *host = self.inner.host_header.clone();
        }

        let domain = self.inner.domain.clone();
        let addr = self.inner.addr;
        let client = self.inner.client.clone();

        Box::pin(async move {
            let mut response = match client.proxy_request(addr, domain, req).await {
                Ok(response) => response,
                Err(err) => {
                    error!("Failed to forward request: {}", err);
                    return Err(err.into());
                }
            };

            if let Some(cookie) = set_cookie.and_then(|cookie| cookie.to_string().parse().ok()) {
                response.headers_mut().append(SET_COOKIE, cookie);
            }

            Ok(response)
        })
    }
}
