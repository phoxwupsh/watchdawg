use base64::{prelude::BASE64_STANDARD, Engine};
use bcrypt::verify;
use dashmap::DashMap;
use std::{io::BufRead, path::Path};
use super::Authenticator;

pub struct HtpasswdAuth {
    credentials: DashMap<String, String>,
}

impl HtpasswdAuth {
    pub fn new(htpasswd_path: impl AsRef<Path>) -> std::io::Result<Self> {
        let file = std::fs::File::open(htpasswd_path)?;
        let reader = std::io::BufReader::new(file);
        let credentials = DashMap::new();

        for line in reader.lines() {
            let line = line?;
            let mut parts = line.split(':');

            let name = parts.next();
            let password = parts.next();

            if let (Some(name), Some(password)) = (name, password) {
                credentials.insert(name.to_string(), password.to_string());
            }
        }
        Ok(Self { credentials })
    }
}

impl Authenticator for HtpasswdAuth {
    fn is_auth_header_valid(&self, auth_header: &[u8]) -> bool {
        let base64_credentials = if auth_header.starts_with(b"Basic ") {
            &auth_header[6..]
        } else {
            return false;
        };
        let Ok(credentials) = BASE64_STANDARD.decode(base64_credentials) else {
            return false;
        };

        let mut parts = credentials.split(|&byte| byte == b':');

        let username = parts.next();
        let password = parts.next();

        if let Some(password) = password {
            if let Some(res) = username
                .and_then(|username| std::str::from_utf8(username).ok())
                .and_then(|username| self.credentials.get(username))
                .and_then(|hash| verify(password, hash.as_str()).ok())
            {
                return res;
            }
        }
        false
    }
}
