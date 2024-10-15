use crate::{auth::Authenticator, session::SessionManager};
use http_body_util::{combinators::BoxBody, BodyExt, Empty};
use hyper::{
    body::Bytes,
    header::{AUTHORIZATION, COOKIE, WWW_AUTHENTICATE},
    HeaderMap, Response, StatusCode,
};
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::{io::BufReader, path::Path};

pub fn req_auth() -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(WWW_AUTHENTICATE, "Basic realm=\"Restricted\"")
        .body(empty())
        .unwrap()
}

pub fn ok_empty() -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder()
        .status(StatusCode::OK)
        .body(empty())
        .unwrap()
}

pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new().map_err(infallible_to_err).boxed()
}

/// Cast [`std::convert::Infallible`] to [`hyper::Error`]
fn infallible_to_err(_: std::convert::Infallible) -> hyper::Error {
    unreachable!()
}

pub fn load_certs(path: impl AsRef<Path>) -> std::io::Result<Vec<CertificateDer<'static>>> {
    let cert_file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(cert_file);
    let certs = rustls_pemfile::certs(&mut reader)
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    Ok(certs)
}

pub fn load_private_key(path: impl AsRef<Path>) -> std::io::Result<PrivateKeyDer<'static>> {
    let key_file = std::fs::File::open(path)?;
    let mut reader = BufReader::new(key_file);
    rustls_pemfile::private_key(&mut reader)?.ok_or(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Invalid private key",
    ))
}

/// Get the session id with the provide cookie name
pub fn get_session_from_cookie<'a>(
    cookie_name: &[u8],
    cookie_header: &'a [u8],
) -> Option<&'a [u8]> {
    for cookie in cookie_header.split(|&part| part == b';') {
        let trimed = if cookie.starts_with(b" ") {
            &cookie[1..]
        } else {
            &cookie[0..]
        };

        if trimed.starts_with(cookie_name) {
            let mut splited = cookie.splitn(2, |&byte| byte == b'=').skip(1); // the first one is cookie name
            return splited.next();
        }
    }
    None
}


/// Return the session id if there is a valid session
pub fn headers_has_valid_session<'a>(
    headers: &'a HeaderMap,
    session_manager: &SessionManager,
) -> Option<&'a str> {
    let session_id = headers
        .get(COOKIE)
        .and_then(|cookie_header| {
            get_session_from_cookie(
                session_manager.cookie_name.as_bytes(),
                cookie_header.as_bytes(),
            )
        })
        .and_then(|session_bytes| std::str::from_utf8(session_bytes).ok())?;
    match session_manager.is_session_valid(session_id) {
        true => Some(session_id),
        false => None,
    }
}

/// Return true if there is a valid authentication
pub fn header_has_valid_auth(headers: &HeaderMap, authenticator: &dyn Authenticator) -> bool {
    headers
        .get(AUTHORIZATION)
        .map(|auth_header| authenticator.is_auth_header_valid(auth_header.as_bytes()))
        .unwrap_or(false)
}
