pub mod htpasswd;

pub trait Authenticator {
    fn is_auth_header_valid(&self, header: &[u8]) -> bool;
}
