use anyhow::{anyhow, bail, Ok, Result};
use bb8_redis::{bb8::Pool, redis::cmd, RedisConnectionManager};
use lazy_static::lazy_static;
use std::{fs::File, net::IpAddr, path::Path, sync::Arc};
use tokio::sync::RwLock;
use tracing::metadata::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

#[cfg(test)]
mod test;

/// trace! macro
///
/// Constructs a new message at the trace level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similary to the trace! macro.
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
                tracing::trace!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
            LogLocation::Both => {
                tracing::trace!($($arg)*);
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
        }
    };
}

/// debug! macro
///
/// Constructs a new message at the debug level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similary to the debug! macro.
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
                tracing::debug!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
            LogLocation::Both => {
                tracing::debug!($($arg)*);
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
        }
    };
}

/// info! macro
///
/// Constructs a new message at the info level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similary to the info! macro.
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
                tracing::info!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
            LogLocation::Both => {
                tracing::info!($($arg)*);
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
        }
    };

}

/// warn! macro
///
/// Constructs a new message at the warn level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similary to the warn! macro.
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
                tracing::warn!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
            LogLocation::Both => {
                tracing::warn!($($arg)*);
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
        }
    };

}

/// error! macro
///
/// Constructs a new message at the error level with the given format string and
/// a desired log location of either to a Redis connection, log file or both.
/// This functions similary to the error! macro.
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
                tracing::error!($($arg)*);
            }
            LogLocation::Redis => {
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
            LogLocation::Both => {
                tracing::error!($($arg)*);
                let log = format!($($arg)*);
                let _ = log_to_redis(&log).await;
            }
        }
    };
}

pub enum LogLocation {
    Local,
    Redis,
    Both,
}

lazy_static! {
    static ref CLIENT_ID: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
    static ref REDIS_POOL: Arc<RwLock<Option<Pool<RedisConnectionManager>>>> =
        Arc::new(RwLock::new(None));
}

/// Init redis connection
///
/// This functions takes in the ip address, port and client id
/// and stores it for later use in a connection pool
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
    let pool = Pool::builder().build(manager).await?;

    let mut pool_lock = REDIS_POOL
        .try_write()
        .map_err(|e| anyhow!("Redis pool error: {}", e))?;
    *pool_lock = Some(pool);

    let mut client_id_lock = CLIENT_ID
        .try_write()
        .map_err(|e| anyhow!("Redis client id error: {}", e))?;
    *client_id_lock = client_id;

    Ok(())
}

#[allow(dead_code)]
async fn log_to_redis(log: &str) -> anyhow::Result<()> {
    let pool_lock = REDIS_POOL.clone();
    let client_id_lock = CLIENT_ID.clone();

    let pool = pool_lock.read().await;
    let client_id = client_id_lock.read().await;

    match pool.clone() {
        Some(pool) => {
            let mut conn = pool.get().await?;
            let _ = cmd("RPUSH")
                .arg(client_id.clone())
                .arg(log.to_owned())
                .query_async::<_, ()>(&mut *conn)
                .await
                .map_err(|e| anyhow!("Can't set log: {}", e));
        }
        None => return Err(anyhow!("Redis pool is not initialized")),
    }

    drop(pool);
    drop(client_id);
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
