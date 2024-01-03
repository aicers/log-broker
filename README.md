# log-broker

A library that supports five logging levels: ERROR, WARNING, INFO, TRACING,
and DEBUG. The log broker forwards these logs to either a Redis server,
to a local log file, or both. When forwarded to a a Redis server, it will
be formatted to a JSON whilst when forwarded to a log file it will be formatted
as a log from the tracing library.

## Usage

```rs
    use log_broker::LogLocation;

    // To initialize tracing to a local file when given a valid path
    // and package name
    // Note: cannot be `_` otherwise the WorkerGuard will immediately
    // be consumed
    let _guard = init_tracing(path, pkg_name)?;

    // To initialize connection to Redis server with default settings
    let _ = init_redis_connection(ip_addr, port, client_id)
        .await
        .map_err(|e| e.to_string());

    // To initialize connection to Redis server with custom settings
    let _ =
        init_redis_connection_with_options(ip_addr, port, client_id.clone(), Some(pool_options))
            .await
            .map_err(|e| e.to_string());

    // Trace log logs to local log file
    trace!(LogLocation::Local, "{}", trace_log);

    // Debug log logs to Redis server initialized above
    debug!(LogLocation::Redis, "{}", debug_log);

    // Info log logs to both local and Redis server
    info!(LogLocation::Both, "{}", info_log);

    // Warn and error macros are also available with either options
    warn!(LogLocation::Both, "{}", warn_log);
    error!(LogLocation::Both, "{}", error_log);

```

## Test

Run log broker while Redis server is running at 127.0.0.1:6379 with:

```sh
cargo test --tests
```

## License

Copyright 2022-2023 ClumL Inc.

Licensed under [Apache License, Version 2.0][apache-license] (the "License");
you may not use this crate except in compliance with the License.

Unless required by applicable law or agreed to in writing, software distributed
under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
CONDITIONS OF ANY KIND, either express or implied. See [LICENSE](LICENSE) for
the specific language governing permissions and limitations under the License.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the [Apache-2.0
license][apache-license], shall be licensed as above, without any additional
terms or conditions.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
