use crate::{
    etl::{
        LogEvent,
        parser::{LogConfig, parsed_with_dynamic_format},
    },
    socket::SharedLogState,
};

use chrono::{DateTime, Timelike, Utc};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{self},
    },
    thread::JoinHandle,
};
use tungstenite::Message;

use serde_json;

pub fn transform_load(
    rx: mpsc::Receiver<LogEvent>,
    config: &Arc<LogConfig>,
    handles: Vec<JoinHandle<()>>,
    etl_socket_clients: SharedLogState,
) {
    analyze_groups(rx, config, etl_socket_clients);

    for handle in handles {
        handle.join().expect("A thread panicked during execution");
    }
}

fn analyze_groups(
    rx: mpsc::Receiver<LogEvent>,
    config: &Arc<LogConfig>,
    etl_socket_clients: SharedLogState,
) {
    let mut grouped_logs: HashMap<DateTime<Utc>, HashMap<String, Vec<String>>> = HashMap::new();
    let lower_indicators: Vec<String> = config
        .error_indicators
        .iter()
        .map(|i| i.to_lowercase())
        .collect();

    while let Ok(event) = rx.recv() {
        if let Some((dt, raw_line)) = parsed_with_dynamic_format(&event.line, &config) {
            let bucket = dt.with_second(0).unwrap().with_nanosecond(0).unwrap();
            grouped_logs
                .entry(bucket)
                .or_insert_with(HashMap::new)
                .entry(event.file.to_string())
                .or_insert_with(Vec::new)
                .push(raw_line);

            //stream to ui
            let bucket_time = dt.format("%H:%M").to_string();
            let msg = serde_json::json!({
                "time": bucket_time,
                "file": event.file.to_string(),
                "line": event.line,
                "error": lower_indicators.iter().any(|ind| event.line.to_lowercase().contains(ind))
            })
            .to_string();
            let mut client_guards = etl_socket_clients.lock().unwrap();
            client_guards.cache.push_back(msg.clone());
            if client_guards.cache.len() > 500 {
                client_guards.cache.pop_front();
            }

            if !client_guards.clients.is_empty() {
                println!("📡 Sending log to {} clients", client_guards.clients.len());
            }
            let msg_bytes: tungstenite::Utf8Bytes = msg.into();
            client_guards
                .clients
                .retain_mut(|ws| ws.send(Message::Text(msg_bytes.clone())).is_ok())
        }
    }

    for (minute, file_logs) in &grouped_logs {
        for (file_name, logs) in file_logs {
            let error_count = logs
                .iter()
                .filter(|line| {
                    let line_lower = line.to_lowercase();
                    lower_indicators.iter().any(|ind| line_lower.contains(ind))
                })
                .count();

            println!(
                "Minute: {} | File: {} | Total Logs: {} | Errors: {} ",
                minute.format("%H:%M"),
                file_name,
                logs.len(),
                error_count,
            );
        }
    }
}
