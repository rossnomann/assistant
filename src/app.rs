use crate::{
    config::{Config, ConfigError},
    handlers,
};
use carapax::{
    access::{AccessExt, AccessRule, InMemoryAccessPolicy},
    longpoll::LongPoll,
    session::{backend::redis::RedisBackend, SessionCollector, SessionManager},
    webhook,
    webhook::HyperError,
    Api, ApiError, App, Context,
};
use redis::{aio::ConnectionManager as RedisConnectionManager, Client as RedisClient, RedisError};
use std::{env, error::Error, fmt, time::Duration};

const SESSION_NAMESPACE: &str = "tg-assistant-bot";
const SESSION_GC_PERIOD: Duration = Duration::from_secs(3_600);
const SESSION_LIFETIME: Duration = Duration::from_secs(86_400 * 30);

pub async fn run() -> Result<(), AppError> {
    let config_path = match env::args().nth(1) {
        Some(path) => path,
        None => return Err(AppError::NoConfig),
    };
    let config = Config::read_from_file(config_path).map_err(AppError::ReadConfig)?;

    let access_rules: Vec<_> = config.users.into_iter().map(AccessRule::allow_user).collect();
    let admin_policy = InMemoryAccessPolicy::from(access_rules);

    let api = Api::new(&config.token).map_err(AppError::CreateApi)?;

    let redis_client = RedisClient::open(config.redis_url).map_err(AppError::Redis)?;
    let redis_manager = RedisConnectionManager::new(redis_client)
        .await
        .map_err(AppError::Redis)?;

    let session_backend = RedisBackend::new(SESSION_NAMESPACE, redis_manager);
    let mut session_collector = SessionCollector::new(session_backend.clone(), SESSION_GC_PERIOD, SESSION_LIFETIME);
    tokio::spawn(async move { session_collector.run().await });

    let session_manager = SessionManager::new(session_backend);

    let mut context = Context::default();
    context.insert(api.clone());
    context.insert(session_manager);

    let chain = handlers::setup().access(admin_policy);

    let app = App::new(context, chain);

    match config.webhook_address {
        Some(address) => {
            let path = config.webhook_path.unwrap_or_else(|| String::from("/"));
            webhook::run_server(address, path, app)
                .await
                .map_err(AppError::StartServer)?;
        }
        None => {
            LongPoll::new(api, app).run().await;
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum AppError {
    CreateApi(ApiError),
    NoConfig,
    ReadConfig(ConfigError),
    Redis(RedisError),
    StartServer(HyperError),
}

impl fmt::Display for AppError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AppError::*;
        match self {
            CreateApi(err) => write!(out, "Could not create API client: {}", err),
            NoConfig => write!(out, "Path to configuration file is not provided"),
            ReadConfig(err) => write!(out, "{}", err),
            Redis(err) => write!(out, "Redis connection error: {}", err),
            StartServer(err) => write!(out, "Could not start server for webhooks: {}", err),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::AppError::*;
        Some(match self {
            CreateApi(err) => err,
            NoConfig => return None,
            ReadConfig(err) => err,
            Redis(err) => err,
            StartServer(err) => err,
        })
    }
}
