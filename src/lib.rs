#[cfg(test)]
mod test;

use anyhow::{anyhow, bail, Ok, Result};
use bb8_redis::{
    bb8::{CustomizeConnection, ErrorSink, ManageConnection, NopErrorSink, Pool, QueueStrategy},
    redis::cmd,
    RedisConnectionManager,
};
use lazy_static::lazy_static;
use std::{fs::File, net::IpAddr, path::Path, sync::Arc, time::Duration};
use tokio::sync::RwLock;
pub use tracing;
use tracing::metadata::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

/// trace! macro
///
/// Constructs a new message at the trace level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similarly to the trace! macro.
///
/// # Examples
///
/// ```text
/// trace!(LogLocation::Local, "trace");
/// trace!(LogLocation::Redis, "trace");
/// trace!(LogLocation::Both, "trace");
/// ```
#[macro_export]
macro_rules! trace {
    ($location:expr,$($arg:tt)*) => {
        match $location {
            LogLocation::Local => {
                $crate::tracing::trace!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("TRACE", &log).await;
                });
            }
            LogLocation::Both => {
                $crate::tracing::trace!($($arg)*);
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("TRACE", &log).await;
                });
            }
        }
    };
}

/// debug! macro
///
/// Constructs a new message at the debug level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similarly to the debug! macro.
///
/// # Examples
///
/// ```text
/// debug!(LogLocation::Local, "debug");
/// debug!(LogLocation::Redis, "debug");
/// debug!(LogLocation::Both, "debug");
/// ```
#[macro_export]
macro_rules! debug {
    ($location:expr,$($arg:tt)*) => {
        match $location {
            LogLocation::Local => {
                $crate::tracing::debug!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("DEBUG", &log).await;
                });
            }
            LogLocation::Both => {
                $crate::tracing::debug!($($arg)*);
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("DEBUG", &log).await;
                });
            }
        }
    };
}

/// info! macro
///
/// Constructs a new message at the info level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similarly to the info! macro.
///
/// # Examples
///
/// ```text
/// info!(LogLocation::Local, "info");
/// info!(LogLocation::Redis, "info");
/// info!(LogLocation::Both, "info);
/// ```
#[macro_export]
macro_rules! info {
    ($location:expr,$($arg:tt)*) => {
        match $location {
            LogLocation::Local => {
                $crate::tracing::info!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("INFO", &log).await;
                });
            }
            LogLocation::Both => {
                $crate::tracing::info!($($arg)*);
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("INFO", &log).await;
                });
            }
        }
    };
}

/// warn! macro
///
/// Constructs a new message at the warn level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similarly to the warn! macro.
///
/// # Examples
///
/// ```text
/// warn!(LogLocation::Local, "warn");
/// warn!(LogLocation::Redis, "warn");
/// warn!(LogLocation::Both, "warn");
/// ```
#[macro_export]
macro_rules! warn {
    ($location:expr,$($arg:tt)*) => {
        match $location {
            LogLocation::Local => {
                $crate::tracing::warn!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("WARN", &log).await;
                });
            }
            LogLocation::Both => {
                $crate::tracing::warn!($($arg)*);
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("WARN", &log).await;
                });
            }
        }
    };
}

/// error! macro
///
/// Constructs a new message at the error level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similarly to the error! macro.
///
/// # Examples
///
/// ```text
/// error!(LogLocation::Local, "error");
/// error!(LogLocation::Redis, "error");
/// error!(LogLocation::Both, "error");
/// ```
#[macro_export]
macro_rules! error {
    ($location:expr,$($arg:tt)*) => {
        match $location {
            LogLocation::Local => {
                $crate::tracing::error!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("ERROR", &log).await;
                });
            }
            LogLocation::Both => {
                $crate::tracing::error!($($arg)*);
                let log = format!($($arg)*);
                tokio::spawn(async move {
                    let _ = $crate::log_to_redis("ERROR", &log).await;
                });
            }
        }
    };
}

pub enum LogLocation {
    Local,
    Redis,
    Both,
}

pub struct RedisConnPoolOptions<M: ManageConnection> {
    max_size: u32,
    min_idle: Option<u32>,
    test_on_check_out: bool,
    max_lifetime: Option<Duration>,
    idle_timeout: Option<Duration>,
    connection_timeout: Duration,
    retry_connection: bool,
    error_sink: Box<dyn ErrorSink<M::Error>>,
    reaper_rate: Duration,
    queue_strategy: QueueStrategy,
    connection_customizer: Option<Box<dyn CustomizeConnection<M::Connection, M::Error>>>,
}

impl<M: ManageConnection> Default for RedisConnPoolOptions<M> {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: None,
            test_on_check_out: false,
            max_lifetime: Some(Duration::from_secs(30 * 60)),
            idle_timeout: Some(Duration::from_secs(10 * 60)),
            connection_timeout: Duration::from_secs(30),
            retry_connection: true,
            error_sink: Box::new(NopErrorSink),
            reaper_rate: Duration::from_secs(30),
            queue_strategy: QueueStrategy::default(),
            connection_customizer: None,
        }
    }
}

impl<M: ManageConnection> RedisConnPoolOptions<M> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn max_size(mut self, max_size: u32) -> Self {
        self.max_size = max_size;
        self
    }

    #[must_use]
    pub fn min_idle(mut self, min_idle: Option<u32>) -> Self {
        self.min_idle = min_idle;
        self
    }

    #[must_use]
    pub fn test_on_check_out(mut self, test_on_check_out: bool) -> Self {
        self.test_on_check_out = test_on_check_out;
        self
    }

    #[must_use]
    pub fn max_lifetime(mut self, max_lifetime: Option<Duration>) -> Self {
        self.max_lifetime = max_lifetime;
        self
    }

    #[must_use]
    pub fn idle_timeout(mut self, idle_timeout: Option<Duration>) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    #[must_use]
    pub fn connection_timeout(mut self, connection_timeout: Duration) -> Self {
        self.connection_timeout = connection_timeout;
        self
    }

    #[must_use]
    pub fn retry_connection(mut self, retry_connection: bool) -> Self {
        self.retry_connection = retry_connection;
        self
    }

    #[must_use]
    pub fn error_sink(mut self, error_sink: Box<dyn ErrorSink<M::Error>>) -> Self {
        self.error_sink = error_sink;
        self
    }

    #[must_use]
    pub fn reaper_rate(mut self, reaper_rate: Duration) -> Self {
        self.reaper_rate = reaper_rate;
        self
    }

    #[must_use]
    pub fn queue_strategy(mut self, queue_strategy: QueueStrategy) -> Self {
        self.queue_strategy = queue_strategy;
        self
    }

    #[must_use]
    pub fn connection_customizer(
        mut self,
        connection_customizer: Box<dyn CustomizeConnection<M::Connection, M::Error>>,
    ) -> Self {
        self.connection_customizer = Some(connection_customizer);
        self
    }
}

lazy_static! {
    static ref CLIENT_ID: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
    static ref REDIS_POOL: Arc<RwLock<Option<Pool<RedisConnectionManager>>>> =
        Arc::new(RwLock::new(None));
}

/// Init redis connection
///
/// This functions takes in the ip address, port, and client id
/// and stores it for later use in a connection pool with default
/// Redis connection pool settings
///
/// # Errors
///
/// * Redis connection error
/// * Redis pool error
/// * Redis client id error
pub async fn init_redis_connection(
    ip_addr: IpAddr,
    port: u16,
    client_id: String,
) -> anyhow::Result<()> {
    let manager = RedisConnectionManager::new(format!("redis://{ip_addr}:{port}"))?;
    let pool = build_pool_with_options::<RedisConnectionManager>(manager, None).await?;

    if pool.get().await.is_ok() {
        {
            let mut pool_lock = REDIS_POOL
                .try_write()
                .map_err(|e| anyhow!("Redis pool error: {}", e))?;
            *pool_lock = Some(pool.clone());
        }

        {
            let mut client_id_lock = CLIENT_ID
                .try_write()
                .map_err(|e| anyhow!("Redis client id error: {}", e))?;
            *client_id_lock = client_id;
        }
        Ok(())
    } else {
        bail!("Redis connection error")
    }
}

/// Init redis connection with custom pool options
///
/// This functions takes in the ip address, port, client id, and pool options
/// and stores it for later use in a connection pool with custom Redis
/// connection pool settings
///
/// # Errors
///
/// * Redis connection error
/// * Redis pool error
/// * Redis client id error
pub async fn init_redis_connection_with_options(
    ip_addr: IpAddr,
    port: u16,
    client_id: String,
    pool_options: Option<RedisConnPoolOptions<RedisConnectionManager>>,
) -> anyhow::Result<()> {
    let manager = RedisConnectionManager::new(format!("redis://{ip_addr}:{port}"))?;
    let pool = build_pool_with_options::<RedisConnectionManager>(manager, pool_options).await?;

    if pool.get().await.is_ok() {
        {
            let mut pool_lock = REDIS_POOL
                .try_write()
                .map_err(|e| anyhow!("Redis pool error: {}", e))?;
            *pool_lock = Some(pool.clone());
        }

        {
            let mut client_id_lock = CLIENT_ID
                .try_write()
                .map_err(|e| anyhow!("Redis client id error: {}", e))?;
            *client_id_lock = client_id;
        }
        Ok(())
    } else {
        bail!("Redis connection error")
    }
}

async fn build_pool_with_options<M: ManageConnection>(
    manager: M,
    pool_options: Option<RedisConnPoolOptions<M>>,
) -> Result<Pool<M>, <M as ManageConnection>::Error> {
    let pool_options = pool_options.unwrap_or_default();
    let mut builder = Pool::builder()
        .max_size(pool_options.max_size)
        .min_idle(pool_options.min_idle)
        .test_on_check_out(pool_options.test_on_check_out)
        .max_lifetime(pool_options.max_lifetime)
        .idle_timeout(pool_options.idle_timeout)
        .connection_timeout(pool_options.connection_timeout)
        .retry_connection(pool_options.retry_connection)
        .error_sink(pool_options.error_sink)
        .reaper_rate(pool_options.reaper_rate)
        .queue_strategy(pool_options.queue_strategy);

    if let Some(connection_customizer) = pool_options.connection_customizer {
        builder = builder.connection_customizer(connection_customizer);
    }

    builder.build(manager).await
}

/// Expanded macro calls this function.
///
/// # Errors
///
/// * Redis error
pub async fn log_to_redis(log_level: &str, log: &str) -> anyhow::Result<()> {
    let now = chrono::Utc::now();
    let formatted = now.format("%Y-%m-%dT%H:%M:%S%.6fZ");
    let log = format!("{formatted}\t{log_level}\t{log}");

    let pool = REDIS_POOL.read().await;

    match &*pool {
        Some(pool) => {
            let mut conn = pool.get().await?;
            let client_id = CLIENT_ID.read().await;
            let _ = cmd("RPUSH")
                .arg(client_id.clone())
                .arg(log)
                .query_async::<_, ()>(&mut *conn)
                .await
                .map_err(|e| anyhow!("Can't set log: {}", e));
        }
        None => return Err(anyhow!("Redis pool is not initialized")),
    }

    Ok(())
}

/// Init operation log with tracing
///
/// This function has parameter `pkg_name` with `env!("CARGO_PKG_NAME")`
///
/// Note the the `WorkerGuard` must be assigned to a binding that is not _.
/// Otherwise, the guard will be dropped immediately and the log file will not be written to.
///
/// # Errors
///
/// * Path not exist
/// * Invalid path
///
pub fn init_tracing(path: &Path, pkg_name: &str) -> Result<WorkerGuard> {
    if !path.exists() {
        tracing_subscriber::fmt::init();
        bail!("Path not found {path:?}");
    }
    let file_name = format!("{pkg_name}.log");
    if File::create(path.join(file_name.clone())).is_err() {
        tracing_subscriber::fmt::init();
        bail!("Cannot create file. {}/{file_name}", path.display());
    }
    let file_appender = tracing_appender::rolling::never(path, file_name);
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    let layer_file = fmt::Layer::default()
        .with_ansi(false)
        .with_target(false)
        .with_writer(file_writer)
        .with_filter(EnvFilter::from_default_env().add_directive(LevelFilter::TRACE.into()));
    let layer_stdout = fmt::Layer::default()
        .with_ansi(true)
        .with_filter(EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(layer_file)
        .with(layer_stdout)
        .init();
    Ok(guard)
}
