use crate::{
    auth::Authenticator,
    session::SessionManager,
    utils::{header_has_valid_auth, headers_has_valid_session, ok_empty, req_auth},
};
use concat_string::concat_string;
use http_body_util::combinators::BoxBody;
use hyper::{
    body::{Bytes, Incoming},
    header::HeaderName,
    service::Service,
    Request, Response,
};
use std::{future::Future, pin::Pin, sync::Arc};

pub mod http;
pub mod https;

#[derive(Clone)]
pub struct AuthOnlySvc {
    inner: Arc<AuthOnlySvcImpl>,
}

struct AuthOnlySvcImpl {
    auth: Arc<dyn Authenticator + Send + Sync + 'static>,
    auth_return_header_name: HeaderName,
    session_manager: SessionManager,
    cookie_generator: Box<dyn Fn(&str) -> String + Send + Sync>,
}

impl AuthOnlySvc {
    pub fn new(
        authenticator: impl Authenticator + Send + Sync + 'static,
        auth_return_header_name: &str,
        session_manager: SessionManager,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let cookie_name_str = session_manager.cookie_name.clone();
        let cookie_generator =
            Box::new(move |session_id: &str| concat_string!(cookie_name_str, "=", session_id));

        let auth_return_header_name =
            HeaderName::from_lowercase(auth_return_header_name.to_ascii_lowercase().as_bytes())?;
        let inner = AuthOnlySvcImpl {
            auth: Arc::new(authenticator),
            auth_return_header_name,
            session_manager,
            cookie_generator,
        };
        Ok(Self {
            inner: inner.into(),
        })
    }
}

impl Service<Request<Incoming>> for AuthOnlySvc {
    type Response = Response<BoxBody<Bytes, hyper::Error>>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let headers = req.headers();
        let mut set_session: Option<String> = None;

        // if there's not valid session
        if headers_has_valid_session(headers, &self.inner.session_manager).is_none() {
            // then check if there's valid authentication
            if header_has_valid_auth(headers, self.inner.auth.as_ref()) {
                // if yes give new session
                let session_id = self.inner.session_manager.create_session();
                set_session = Some((self.inner.cookie_generator)(&session_id));
            } else {
                // else return unauthroized and reqest authentication
                return Box::pin(async move { Ok(req_auth()) });
            }
        }
        let auth_return_header_name = self.inner.auth_return_header_name.clone();
        Box::pin(async move {
            let mut resp = ok_empty();
            if let Some(session) = set_session.and_then(|session| session.parse().ok()) {
                resp.headers_mut().insert(auth_return_header_name, session);
            }
            Ok(resp)
        })
    }
}
