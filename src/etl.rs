pub mod parser;
use parser::LogConfig;
use std::sync::Arc;

pub mod transform_load;
use transform_load::transform_load;

pub mod extract;
use extract::extract;
pub struct LogEvent {
    file: Arc<str>,
    line: String,
}
use crate::socket::SharedLogState;

pub fn etl(
    path: &str,
    config: Arc<LogConfig>,
    file_paths: Vec<String>,
    etl_socket_clients: SharedLogState,
) {
    let (rx, handles) = extract(path, file_paths);
    transform_load(rx, &config, handles, etl_socket_clients);
}
