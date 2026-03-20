use futures::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use gloo_timers::future::TimeoutFuture;
use serde::Deserialize;
use std::collections::{BTreeMap, VecDeque};
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys;
#[derive(Deserialize, Debug, Clone)]
struct LogUpdate {
    time: String,
    file: String,
    line: String,
    error: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct MetricUpdate {
    time: String,
    cpu_usage: f64,
    ram_total: u64,
    ram_used: u64,
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_name = updateLogUPlot)]
    fn update_log_uplot(x: JsValue, y_total: JsValue, y_errors: JsValue, timestamps: JsValue);

    #[wasm_bindgen(js_name = updateMetricUPlot)]
    fn update_metric_uplot(x: JsValue, y_total: JsValue, y_errors: JsValue, timestamps: JsValue);
}

#[wasm_bindgen]
pub fn greet_analyzer(name: &str) {
    let message = format!("Hello from the Log Analyzer WASM, {}!", name);
    alert(&message);
}

#[wasm_bindgen]
pub fn start_log_stream() {
    spawn_local(async {
        let mut ws = WebSocket::open("ws://127.0.0.1:9001").expect("Websocket failed to connect");
        let mut log_bucket_counts: BTreeMap<String, u32> = BTreeMap::new();
        let mut log_error_counts: BTreeMap<String, u32> = BTreeMap::new();

        let mut metric_timestamps: VecDeque<String> = VecDeque::with_capacity(500);
        let mut cpu_history: VecDeque<f64> = VecDeque::with_capacity(500);
        let mut ram_history: VecDeque<f64> = VecDeque::with_capacity(500);
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(log) = serde_json::from_str::<LogUpdate>(&text) {
                        render_log_to_dom(&log);

                        let full_ts = log.time.clone();
                        let mut minute_ts = full_ts.clone();
                        if minute_ts.len() >= 8 {
                            minute_ts.replace_range(6..8, "00");
                        }

                        *log_bucket_counts.entry(minute_ts.clone()).or_insert(0) += 1;
                        if log.error {
                            *log_error_counts.entry(minute_ts.clone()).or_insert(0) += 1;
                        }

                        if log_bucket_counts.len() > 500 {
                            if let Some(first_key) = log_bucket_counts.keys().next().cloned() {
                                log_bucket_counts.remove(&first_key);
                                log_error_counts.remove(&first_key);
                            }
                        }

                        let sorted_timestamps: Vec<String> =
                            log_bucket_counts.keys().cloned().collect();
                        let y_total: Vec<u32> = log_bucket_counts.values().cloned().collect();

                        let y_errors: Vec<u32> = sorted_timestamps
                            .iter()
                            .map(|ts| *log_error_counts.get(ts).unwrap_or(&0))
                            .collect();

                        let x_axis: Vec<usize> = (0..sorted_timestamps.len()).collect();
                        update_log_uplot(
                            serde_wasm_bindgen::to_value(&x_axis).unwrap(),
                            serde_wasm_bindgen::to_value(&y_total).unwrap(),
                            serde_wasm_bindgen::to_value(&y_errors).unwrap(),
                            serde_wasm_bindgen::to_value(&sorted_timestamps).unwrap(),
                        );

                        update_log_text_counters(&log_bucket_counts, &log_error_counts);
                    }
                    if let Ok(metric) = serde_json::from_str::<MetricUpdate>(&text) {
                        let ram_pct = if metric.ram_total > 0 {
                            (metric.ram_used as f64 / metric.ram_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        if metric_timestamps.len() >= 500 {
                            metric_timestamps.pop_front();
                            cpu_history.pop_front();
                            ram_history.pop_front();
                        }

                        metric_timestamps.push_back(metric.time.clone());
                        cpu_history.push_back(metric.cpu_usage);
                        ram_history.push_back(ram_pct);

                        let x_axis: Vec<usize> = (0..metric_timestamps.len()).collect();

                        update_metric_uplot(
                            serde_wasm_bindgen::to_value(&x_axis).unwrap(),
                            serde_wasm_bindgen::to_value(&cpu_history).unwrap(),
                            serde_wasm_bindgen::to_value(&ram_history).unwrap(),
                            serde_wasm_bindgen::to_value(&metric_timestamps).unwrap(),
                        );

                        update_metric_text_labels(metric.cpu_usage, ram_pct);
                    }
                    TimeoutFuture::new(500).await;
                }
                Err(e) => {
                    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!(
                        "WebSocket error: {:?}",
                        e
                    )));
                    break;
                }
                _ => {}
            }
        }
    })
}

fn render_log_to_dom(log: &LogUpdate) {
    let document = web_sys::window().unwrap().document().unwrap();
    if let Some(container) = document.get_element_by_id("log-container") {
        let val = document.create_element("div").unwrap();
        let text_color = if log.error { "red" } else { "#d1d5db" };

        val.set_inner_html(&format!(
            "<div style='color: {color}; font-family: monospace; border-bottom: 1px solid #222;'>
                    <span style='opacity: 0.5;'>[{time}]</span> <b>{file}</b>: {line}
                </div>",
            color = text_color,
            time = log.time,
            file = log.file,
            line = log.line
        ));
        container.prepend_with_node_1(&val).unwrap();

        if container.child_element_count() > 500 {
            let _ = container
                .last_element_child()
                .map(|el| container.remove_child(&el));
        }
    }
}

fn update_log_text_counters(total_map: &BTreeMap<String, u32>, error_map: &BTreeMap<String, u32>) {
    let document = web_sys::window().unwrap().document().unwrap();

    if let Some(el) = document.get_element_by_id("total-logs") {
        el.set_text_content(Some(&total_map.values().sum::<u32>().to_string()));
    }
    if let Some(el) = document.get_element_by_id("error-count") {
        el.set_text_content(Some(&error_map.values().sum::<u32>().to_string()));
    }
}

fn update_metric_text_labels(cpu: f64, ram: f64) {
    let document = web_sys::window().unwrap().document().unwrap();
    if let Some(el) = document.get_element_by_id("cpu-current") {
        el.set_text_content(Some(&format!("{:.1}%", cpu)));
    }
    if let Some(el) = document.get_element_by_id("ram-current") {
        el.set_text_content(Some(&format!("{:.1}%", ram)));
    }
}
