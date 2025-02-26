use std::{error::Error, fmt, io::Error as IoError, sync::Arc};

use carapax::{
    App, Context,
    access::{AccessExt, AccessRule, InMemoryAccessPolicy},
    api::{Client, ClientError},
    handler::{LongPoll, WebhookServer},
    session::SessionManager,
};
use clap::{Parser, Subcommand};
use redis::RedisError;
use refinery::Error as MigrationError;
use tokio::spawn;
use tokio_postgres::{Client as PgClient, Error as PgError, NoTls as PgNoTls, connect as pg_connect};

use crate::{
    config::{Config, ConfigError},
    handlers, migrations,
    services::NotesService,
    session::create_session_backend,
};

#[derive(Parser)]
#[clap(about, author, version)]
pub struct Arguments {
    /// Command to run
    #[clap(subcommand)]
    command: Command,
    /// Path to config
    config: String,
}

#[derive(Subcommand)]
pub enum Command {
    /// Run migrations
    Migrate,
    /// Start bot
    Start,
}

pub async fn run() -> Result<(), AppError> {
    let args = Arguments::parse();

    let config = Config::read_from_file(args.config).map_err(AppError::ReadConfig)?;

    let (mut pg_client, pg_connection) = pg_connect(&config.database_url, PgNoTls)
        .await
        .map_err(AppError::PgConnect)?;

    spawn(async move {
        if let Err(err) = pg_connection.await {
            log::error!("PostgreSQL connection error: {}", err);
        }
    });

    match args.command {
        Command::Migrate => {
            migrations::run(&mut pg_client).await.map_err(AppError::Migrate)?;
        }
        Command::Start => {
            start(config, pg_client).await?;
        }
    }

    Ok(())
}

async fn start(config: Config, pg_client: PgClient) -> Result<(), AppError> {
    let pg_client = Arc::new(pg_client);

    let access_rules: Vec<_> = config.users.into_iter().map(AccessRule::allow_user).collect();
    let admin_policy = InMemoryAccessPolicy::from(access_rules);

    let client = Client::new(&config.token).map_err(AppError::CreateApiClient)?;

    let session_backend = create_session_backend(config.session_url)
        .await
        .map_err(AppError::Redis)?;

    let session_manager = SessionManager::new(session_backend);

    let mut context = Context::default();
    context.insert(client.clone());
    context.insert(session_manager);
    context.insert(NotesService::new(pg_client));

    let chain = handlers::setup().with_access_policy(admin_policy);

    let app = App::new(context, chain);

    match config.webhook_address {
        Some(address) => {
            let path = config.webhook_path.unwrap_or_else(|| String::from("/"));
            WebhookServer::new(path, app)
                .run(address)
                .await
                .map_err(AppError::StartServer)?;
        }
        None => {
            LongPoll::new(client, app).run().await;
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum AppError {
    CreateApiClient(ClientError),
    Migrate(MigrationError),
    NoConfig,
    PgConnect(PgError),
    ReadConfig(ConfigError),
    Redis(RedisError),
    StartServer(IoError),
}

impl fmt::Display for AppError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AppError::*;
        match self {
            CreateApiClient(err) => write!(out, "Could not create API client: {err}"),
            Migrate(err) => write!(out, "Migration error: {err}"),
            NoConfig => write!(out, "Path to configuration file is not provided"),
            PgConnect(err) => write!(out, "PostgreSQL: {err}"),
            ReadConfig(err) => write!(out, "{err}"),
            Redis(err) => write!(out, "Redis connection error: {err}"),
            StartServer(err) => write!(out, "Could not start server for webhooks: {err}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::AppError::*;
        Some(match self {
            CreateApiClient(err) => err,
            Migrate(err) => err,
            NoConfig => return None,
            PgConnect(err) => err,
            ReadConfig(err) => err,
            Redis(err) => err,
            StartServer(err) => err,
        })
    }
}
