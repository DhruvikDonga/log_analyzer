use crate::socket::HubMsg;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::helper::get_ist_time;
use std::sync::mpsc::Sender;
use std::thread;

pub fn get_metrics(metric_tx: Sender<HubMsg>) {
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

        let _ = metric_tx.send(HubMsg::MetricData(msg));

        thread::sleep(std::time::Duration::from_secs(1));
    }
}
