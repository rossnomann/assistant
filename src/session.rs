use carapax::session::{backend::redis::RedisBackend, SessionCollector};
use redis::{aio::ConnectionManager, Client, IntoConnectionInfo, RedisError};
use std::time::Duration;

const SESSION_NAMESPACE: &str = "tg-assistant-bot";
const SESSION_GC_PERIOD: Duration = Duration::from_secs(3_600);
const SESSION_LIFETIME: Duration = Duration::from_secs(86_400 * 30);

pub type SessionBackend = RedisBackend<ConnectionManager>;

pub async fn create_session_backend<T: IntoConnectionInfo>(session_url: T) -> Result<SessionBackend, RedisError> {
    let redis_client = Client::open(session_url)?;
    let redis_manager = ConnectionManager::new(redis_client).await?;
    let backend = RedisBackend::new(SESSION_NAMESPACE, redis_manager);
    let mut collector = SessionCollector::new(backend.clone(), SESSION_GC_PERIOD, SESSION_LIFETIME);
    tokio::spawn(async move { collector.run().await });
    Ok(backend)
}
