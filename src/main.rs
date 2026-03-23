use clap::Parser;
use serde::Deserialize;
use std::{fs, net::TcpListener, sync::Arc, thread};
mod etl;
use etl::{etl, parser::LogConfig};
mod helper;
use helper::ThreadPool;
use std::collections::VecDeque;
mod web;
use rust_embed::RustEmbed;
use web::handle_connection;
mod socket;
use socket::LogState;

mod sys_metrics;
use std::sync::Mutex;

use crate::socket::SharedLogState;

#[derive(RustEmbed)]
#[folder = "wasm_client/site/dist/"]
struct Asset;

#[derive(Parser)]
struct Args {
    /// Path of the directory
    #[arg(short, long)]
    dir_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FilesYAMLConfig {
    files: Vec<String>,
}

fn main() {
    let clients: SharedLogState = Arc::new(Mutex::new(LogState {
        log_cache: VecDeque::with_capacity(500),
        metric_cache: VecDeque::with_capacity(500),
    }));
    let hub_tx = socket::start_broadcaster(Arc::clone(&clients));

    let args = Args::parse();
    println!("Log Analyzer");
    if args.dir_path.is_none() {
        println!("Using files.yaml files");
    }
    let config_data = fs::read_to_string("formats.json").expect("Unable to read formats.json");

    let config: Arc<LogConfig> = Arc::new(serde_json::from_str(&config_data).unwrap());

    socket::start_socket_server(hub_tx.clone());

    let metric_tx = hub_tx.clone();
    let metric_handle = thread::spawn(move || {
        sys_metrics::get_metrics(metric_tx);
    });

    let etl_config = Arc::clone(&config);
    let etl_tx = hub_tx.clone();
    let etl_handle = thread::spawn(move || {
        if args.dir_path.is_none() {
            let file = fs::File::open("files.yaml").expect("Fail to read YAML file");
            let yaml_config: FilesYAMLConfig =
                serde_yml::from_reader(file).expect("Fail to process YAML file");

            if !yaml_config.files.is_empty() {
                etl("", etl_config, yaml_config.files, etl_tx);
            }
        } else {
            if let Some(path) = args.dir_path.as_ref() {
                let file_count = fs::read_dir(path)
                    .unwrap()
                    .flatten()
                    .filter(|entry: &std::fs::DirEntry| entry.path().is_file())
                    .count();

                if file_count > 0 {
                    etl(path, etl_config, Vec::new(), etl_tx);
                }
            }
        }
    });

    let web_handle = thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:7867").unwrap();
        let pool = ThreadPool::new(4);
        println!("Starting web server at port 7867");
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            pool.execute(|| handle_connection(stream));
        }
    });

    etl_handle.join().unwrap();
    metric_handle.join().unwrap();
    web_handle.join().unwrap();
}
