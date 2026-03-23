use std::sync::{Arc, Mutex};
use tungstenite::{self, Message};

use std::collections::VecDeque;
use std::thread;

pub struct LogState {
    pub clients: Vec<tungstenite::WebSocket<std::net::TcpStream>>,
    pub log_cache: VecDeque<String>,
    pub metric_cache: VecDeque<String>,
}
pub type SharedLogState = Arc<Mutex<LogState>>;

pub fn start_socket_server(clients: SharedLogState) {
    let server = std::net::TcpListener::bind("127.0.0.1:9001").unwrap();
    println!("Websocket server started at 9001");

    thread::spawn(move || {
        for s in server.incoming().flatten() {
            if let Ok(mut ws) = tungstenite::accept(s)
                && let Ok(mut guard) = clients.lock()
            {
                for old_logs in &guard.log_cache {
                    let _ = ws.send(Message::Text(old_logs.clone().into()));
                    //thread::sleep(std::time::Duration::from_secs(1));
                }

                for metric in &guard.metric_cache {
                    let _ = ws.send(Message::Text(metric.clone().into()));
                }
                guard.clients.push(ws);
            }
        }
    });
}
