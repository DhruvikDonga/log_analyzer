use crate::socket::SharedLogState;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::helper::get_ist_time;
use serde_json;
use std::thread;
use std::time::Duration;
use tungstenite::Message;

pub fn get_metrics(etl_socket_clients: SharedLogState) {
    println!("Starting metrics collection");
    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything()),
    );
    loop {
        sys.refresh_cpu();
        sys.refresh_memory();
        let bucket_time = get_ist_time(None);

        let msg = serde_json::json!({
            "cpu_usage": sys.global_cpu_info().cpu_usage(),
            "ram_total": sys.total_memory(),
            "ram_used": sys.used_memory(),
            "time": bucket_time
        })
        .to_string();

        let mut metric_clients = etl_socket_clients.lock().unwrap();
        metric_clients.metric_cache.push_back(msg.clone());
        if metric_clients.metric_cache.len() > 500 {
            metric_clients.metric_cache.pop_front();
        }

        let msg_bytes: tungstenite::Utf8Bytes = msg.into();
        metric_clients
            .clients
            .retain_mut(|ws| ws.send(Message::Text(msg_bytes.clone())).is_ok());

        thread::sleep(Duration::from_secs(2));
    }
}
