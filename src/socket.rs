use std::sync::{Arc, Mutex};
use tungstenite::{self, Message};

use std::collections::VecDeque;
use std::thread;

pub struct LogState {
    pub log_cache: VecDeque<String>,
    pub metric_cache: VecDeque<String>,
}
pub enum HubMsg {
    NewClient(tungstenite::WebSocket<std::net::TcpStream>),
    LogData(String),
    MetricData(String),
}
pub type SharedLogState = Arc<Mutex<LogState>>;

pub fn start_socket_server(tx: std::sync::mpsc::Sender<HubMsg>) {
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    println!("Websocket server started at 9001");

    thread::spawn(move || {
        for stream in server.incoming().flatten() {
            if let Ok(ws) = tungstenite::accept(stream) {
                let _ = tx.send(HubMsg::NewClient(ws));
                // Instant! No waiting for logs to send.
            }
        }
    });
}

pub fn start_broadcaster(state: SharedLogState) -> std::sync::mpsc::Sender<HubMsg> {
    let (tx, rx) = std::sync::mpsc::channel::<HubMsg>();

    thread::spawn(move || {
        let mut active_clients = Vec::new();

        while let Ok(msg) = rx.recv() {
            match msg {
                HubMsg::NewClient(mut ws) => {
                    let (logs, metrics) = {
                        let guard = state.lock().unwrap();
                        (guard.log_cache.clone(), guard.metric_cache.clone())
                    };

                    for log in logs {
                        let _ = ws.send(Message::Text(log.into()));
                    }
                    for m in metrics {
                        let _ = ws.send(Message::Text(m.into()));
                    }

                    active_clients.push(ws);
                }
                HubMsg::LogData(raw_text) => {
                    {
                        let mut guard = state.lock().unwrap();
                        guard.log_cache.push_back(raw_text.clone());
                        if guard.log_cache.len() > 500 {
                            guard.log_cache.pop_front();
                        }
                    }
                    let msg_bytes: tungstenite::Utf8Bytes = raw_text.into();
                    // Broadcast to everyone
                    active_clients
                        .retain_mut(|ws| ws.send(Message::Text(msg_bytes.clone())).is_ok());
                }
                HubMsg::MetricData(raw_text) => {
                    {
                        let mut guard = state.lock().unwrap();
                        guard.metric_cache.push_back(raw_text.clone());
                        if guard.metric_cache.len() > 500 {
                            guard.metric_cache.pop_front();
                        }
                    }
                    let msg_bytes: tungstenite::Utf8Bytes = raw_text.into();
                    // Broadcast to everyone
                    active_clients
                        .retain_mut(|ws| ws.send(Message::Text(msg_bytes.clone())).is_ok());
                }
            }
        }
    });

    tx
}
