use futures::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use gloo_timers::future::TimeoutFuture;
use serde::Deserialize;
use std::collections::BTreeMap;
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

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_name = updateUPlot)]
    fn update_uplot(x: JsValue, y_total: JsValue, y_errors: JsValue);
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
        let mut bucket_counts: BTreeMap<i32, u32> = BTreeMap::new();
        let mut error_counts: BTreeMap<i32, u32> = BTreeMap::new();
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(log) = serde_json::from_str::<LogUpdate>(&text) {
                        render_log_to_dom(&log);
                        if let Some(minute_str) = log.time.split(':').last() {
                            if let Ok(m) = minute_str.parse::<i32>() {
                                *bucket_counts.entry(m).or_insert(0) += 1;

                                if log.error {
                                    *error_counts.entry(m).or_insert(0) += 1;
                                }

                                let x_axis: Vec<i32> = bucket_counts.keys().cloned().collect();
                                let y_total: Vec<u32> = bucket_counts.values().cloned().collect();
                                let y_errors: Vec<u32> = x_axis
                                    .iter()
                                    .map(|m| *error_counts.get(m).unwrap_or(&0))
                                    .collect();

                                println!(
                                    "x_axis: {:?}, y_total: {:?}, y_errors: {:?}",
                                    x_axis, y_total, y_errors
                                );
                                update_uplot(
                                    serde_wasm_bindgen::to_value(&x_axis).unwrap(),
                                    serde_wasm_bindgen::to_value(&y_total).unwrap(),
                                    serde_wasm_bindgen::to_value(&y_errors).unwrap(),
                                );
                                TimeoutFuture::new(500).await;

                                update_text_counters(&bucket_counts, &error_counts);
                            }
                        }
                    }
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

fn update_text_counters(total_map: &BTreeMap<i32, u32>, error_map: &BTreeMap<i32, u32>) {
    let document = web_sys::window().unwrap().document().unwrap();

    if let Some(el) = document.get_element_by_id("total-logs") {
        el.set_text_content(Some(&total_map.values().sum::<u32>().to_string()));
    }
    if let Some(el) = document.get_element_by_id("error-count") {
        el.set_text_content(Some(&error_map.values().sum::<u32>().to_string()));
    }
}
