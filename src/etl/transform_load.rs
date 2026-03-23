use crate::{
    etl::{
        LogEvent,
        parser::{LogConfig, parsed_with_dynamic_format},
    },
    helper::get_ist_time,
    socket::HubMsg,
};

use chrono::{DateTime, Timelike, Utc};
use serde_json;
use std::sync::mpsc::Sender;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{self},
    },
    thread::JoinHandle,
};

pub fn transform_load(
    rx: mpsc::Receiver<LogEvent>,
    config: &Arc<LogConfig>,
    handles: Vec<JoinHandle<()>>,
    etl_tx: Sender<HubMsg>,
) {
    analyze_groups(rx, config, etl_tx);

    for handle in handles {
        handle.join().expect("A thread panicked during execution");
    }
}

fn analyze_groups(rx: mpsc::Receiver<LogEvent>, config: &Arc<LogConfig>, etl_tx: Sender<HubMsg>) {
    let mut grouped_logs: HashMap<DateTime<Utc>, HashMap<String, Vec<String>>> = HashMap::new();
    let lower_indicators: Vec<String> = config
        .error_indicators
        .iter()
        .map(|i| i.to_lowercase())
        .collect();

    while let Ok(event) = rx.recv() {
        if let Some((dt, raw_line)) = parsed_with_dynamic_format(&event.line, config) {
            let bucket = dt.with_second(0).unwrap().with_nanosecond(0).unwrap();
            grouped_logs
                .entry(bucket)
                .or_default()
                .entry(event.file.to_string())
                .or_default()
                .push(raw_line);

            // 2. Prepare JSON
            let bucket_time = get_ist_time(Some(dt));
            let is_error = lower_indicators
                .iter()
                .any(|ind| event.line.to_lowercase().contains(ind));

            let msg = serde_json::json!({
                "time": bucket_time,
                "file": event.file.to_string(),
                "line": event.line,
                "error": is_error
            })
            .to_string();

            let _ = etl_tx.send(HubMsg::LogData(msg));
        }
    }
}
