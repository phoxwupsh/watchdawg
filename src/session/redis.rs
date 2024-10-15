use super::SessionStore;
use r2d2::{ManageConnection, Pool, PooledConnection};
use redis::{
    cmd, Commands, Connection, ConnectionInfo, ConnectionLike, IntoConnectionInfo, RedisError,
};
use tracing::error;

pub struct RedisStore {
    pool: Pool<RedisConnPool>,
}

impl RedisStore {
    pub fn new(conn_str: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let manager = RedisConnPool::new(conn_str)?;
        let pool = Pool::builder().build(manager)?;
        Ok(Self { pool })
    }

    fn get_conn(&self) -> Option<PooledConnection<RedisConnPool>> {
        match self.pool.get() {
            Ok(conn) => Some(conn),
            Err(err) => {
                error!("Failed to set redis value: {}", err);
                None
            }
        }
    }
}

impl SessionStore for RedisStore {
    fn save(&self, session_id: &str, timestamp: u64) -> Option<()> {
        let mut conn = self.get_conn()?;
        if let Err(err) = conn.set::<&str, u64, ()>(session_id, timestamp) {
            error!("Failed to set redis value: {}", err);
            return None;
        }
        Some(())
    }
    fn load(&self, session_id: &str) -> Option<u64> {
        let mut conn = self.get_conn()?;
        conn.get(session_id).ok()
    }
    fn delete(&self, session_id: &str) -> Option<u64> {
        let mut conn = self.get_conn()?;
        match conn.get_del(session_id) {
            Ok(res) => Some(res),
            Err(err) => {
                error!("Failed to delete redis value: {}", err);
                None
            }
        }
    }
}

struct RedisConnPool {
    conn_info: ConnectionInfo,
}

impl RedisConnPool {
    fn new(conn_info: impl IntoConnectionInfo) -> Result<Self, RedisError> {
        Ok(Self {
            conn_info: conn_info.into_connection_info()?,
        })
    }
}

impl ManageConnection for RedisConnPool {
    type Connection = Connection;
    type Error = RedisError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        redis::Client::open(self.conn_info.clone())?.get_connection()
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        cmd("PING").query(conn)
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        !conn.is_open()
    }
}
