use clap::Parser;
use serde::Deserialize;
use serde_yml;
use std::{fs, net::TcpListener, sync::Arc, thread};
mod etl;
use etl::{etl, parser::LogConfig};
use log_analyzer::ThreadPool;
use std::collections::VecDeque;
mod web;
use rust_embed::RustEmbed;
use web::handle_connection;
mod socket;
use socket::{LogState, start_socket_server};

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
        clients: Vec::new(),
        cache: VecDeque::with_capacity(500),
    }));
    let args = Args::parse();
    println!("Log Analyzer");
    if args.dir_path.is_none() {
        println!("Using files.yaml files");
    }
    let config_data = fs::read_to_string("formats.json").expect("Unable to read formats.json");

    let config: Arc<LogConfig> = Arc::new(serde_json::from_str(&config_data).unwrap());
    let etl_config = Arc::clone(&config);
    let etl_clients = Arc::clone(&clients);
    let etl_handle = thread::spawn(move || {
        if args.dir_path.is_none() {
            let file = fs::File::open("files.yaml").expect("Fail to read YAML file");
            let yaml_config: FilesYAMLConfig =
                serde_yml::from_reader(file).expect("Fail to process YAML file");

            if !yaml_config.files.is_empty() {
                etl("", etl_config, yaml_config.files, etl_clients);
            }
        } else {
            if let Some(path) = args.dir_path.as_ref() {
                let file_count = fs::read_dir(path)
                    .unwrap()
                    .flatten()
                    .filter(|entry: &std::fs::DirEntry| entry.path().is_file())
                    .count();

                if file_count > 0 {
                    etl(path, etl_config, Vec::new(), etl_clients);
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

    start_socket_server(Arc::clone(&clients));
    etl_handle.join().unwrap();
    web_handle.join().unwrap();
}
