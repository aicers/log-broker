use crate::{
    debug, error, info, init_redis_connection, init_tracing, log_to_redis, trace, warn,
    LogLocation, CLIENT_ID, REDIS_POOL,
};
use anyhow::{anyhow, Result};
use bb8_redis::redis::cmd;
use std::{net::IpAddr, path::Path};

#[tokio::test]
async fn test_init_redis_connection() {
    let ip_addr = IpAddr::from([127, 0, 0, 1]);
    let port = 6379;
    let client_id = "test_connection".to_string();
    let _ = init_redis_connection(ip_addr, port, client_id)
        .await
        .map_err(|e| e.to_string());
    let pool = REDIS_POOL.read().await;

    assert!(pool.is_some());
    let client_id = CLIENT_ID.read().await;
    assert_eq!(*client_id, "test_connection".to_string());
}

#[tokio::test]
async fn test_log_to_redis() -> Result<()> {
    let ip_addr = IpAddr::from([127, 0, 0, 1]);
    let port = 6379;
    let client_id = "test_log_to_redis".to_string();
    let _ = init_redis_connection(ip_addr, port, client_id)
        .await
        .map_err(|e| e.to_string());
    let log = "Successfully Logged".to_string();
    let pool = REDIS_POOL.read().await;
    let client_id = CLIENT_ID.read().await;

    match pool.clone() {
        Some(pool) => {
            let mut conn = pool.get().await?;
            let _ = cmd("DEL")
                .arg(client_id.clone())
                .query_async::<_, ()>(&mut *conn)
                .await?;
            let _ = log_to_redis(&log).await.map_err(|e| e.to_string());
            let test = cmd("LRANGE")
                .arg(client_id.clone())
                .arg(0)
                .arg(-1)
                .query_async::<_, Vec<String>>(&mut *conn)
                .await?;

            assert_eq!(test, vec!["Successfully Logged".to_string()]);
            Ok(())
        }
        None => Err(anyhow!("Redis pool is not initialized")),
    }
}

#[tokio::test]
async fn test_macros() -> Result<()> {
    let path = Path::new("./");
    let pkg_name = "test_macros";
    let _guard = init_tracing(path, pkg_name)?;
    let ip_addr = IpAddr::from([127, 0, 0, 1]);
    let port = 6379;
    let client_id = "test_macros".to_string();
    let _ = init_redis_connection(ip_addr, port, client_id)
        .await
        .map_err(|e| e.to_string());
    let pool = REDIS_POOL.read().await;
    let client_id = CLIENT_ID.read().await;

    match pool.clone() {
        Some(pool) => {
            let mut conn = pool.get().await?;
            let _ = cmd("DEL")
                .arg(client_id.clone())
                .query_async::<_, ()>(&mut *conn)
                .await?;
        }
        None => (),
    }

    let trace_log = "trace".to_string();
    trace!(LogLocation::Local, "{}", trace_log);
    trace!(LogLocation::Redis, "{}", trace_log);
    trace!(LogLocation::Both, "{}", trace_log);

    let debug_log = "debug".to_string();
    debug!(LogLocation::Local, "{}", debug_log);
    debug!(LogLocation::Redis, "{}", debug_log);
    debug!(LogLocation::Both, "{}", debug_log);

    let info_log = "info".to_string();
    info!(LogLocation::Local, "{}", info_log);
    info!(LogLocation::Redis, "{}", info_log);
    info!(LogLocation::Both, "{}", info_log);

    let warn_log = "warn".to_string();
    warn!(LogLocation::Local, "{}", warn_log);
    warn!(LogLocation::Redis, "{}", warn_log);
    warn!(LogLocation::Both, "{}", warn_log);

    let error_log = "error".to_string();
    error!(LogLocation::Local, "{}", error_log);
    error!(LogLocation::Redis, "{}", error_log);
    error!(LogLocation::Both, "{}", error_log);

    Ok(())
}
