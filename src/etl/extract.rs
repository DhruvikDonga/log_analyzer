use crate::etl::LogEvent;
use notify::{Config, RecursiveMode, Watcher};
use std::fs::File;
use std::io::{BufRead, Seek};
use std::{
    fs,
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread,
    thread::JoinHandle,
};

pub fn extract(
    dir_path: &str,
    file_paths: Vec<String>,
) -> (Receiver<LogEvent>, Vec<JoinHandle<()>>) {
    let (tx, rx) = mpsc::sync_channel(1000);

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    if dir_path.len() > 0 {
        for entry in fs::read_dir(dir_path).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                let tx_clone = tx.clone();
                let file_path: Arc<str> = Arc::from(entry.path().to_string_lossy().to_string());

                let handle = thread::spawn(move || {
                    tail_logs(file_path, tx_clone);
                });
                handles.push(handle);
            }
        }
    } else {
        for file_path in file_paths {
            let tx_clone = tx.clone();
            let file_path: Arc<str> = Arc::from(file_path.to_string());

            let handle = thread::spawn(move || {
                tail_logs(file_path, tx_clone);
            });
            handles.push(handle);
        }
    }
    drop(tx);

    return (rx, handles);
}

fn tail_logs(file_path: Arc<str>, tx_clone: mpsc::SyncSender<LogEvent>) {
    print!("Processing file: {} \n", file_path);
    let path = std::path::Path::new(&*file_path);
    let mut file = File::open(path).expect("Error in opening the file");
    let mut reader = std::io::BufReader::new(file);
    let mut line = String::new();

    while let Ok(bytes) = reader.read_line(&mut line) {
        if bytes == 0 {
            break;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }
        let log_event = LogEvent {
            file: file_path.clone(),
            line: line.trim().to_string(),
        };
        tx_clone.send(log_event).ok();
        line.clear();
    }

    let (event_tx, event_rx) = mpsc::channel();

    let mut watcher =
        notify::RecommendedWatcher::new(event_tx, Config::default()).expect("Watcher failed");

    watcher
        .watch(path, RecursiveMode::NonRecursive)
        .expect("Watch failed");

    for res in event_rx {
        match res {
            Ok(event) => {
                let mut should_read = event.kind.is_modify();
                if event.kind.is_remove() || event.kind.is_modify() {
                    if let Ok(meta) = std::fs::metadata(&path) {
                        let current_pos = reader.stream_position().unwrap_or(0);
                        if meta.len() < current_pos {
                            file =
                                File::open(path).expect("Error in re-opening the truncated file");
                            reader = std::io::BufReader::new(file);
                            should_read = true;
                        }
                    } else {
                        //file was removed or does not exist
                        // do not read from it
                        //try to open it again
                        thread::sleep(std::time::Duration::from_millis(100));
                        if let Ok(f) = File::open(&path) {
                            file = f;
                            reader = std::io::BufReader::new(file);
                            should_read = true;
                        }
                    }
                }
                if should_read {
                    while let Ok(bytes) = reader.read_line(&mut line) {
                        if bytes == 0 {
                            break;
                        }
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            line.clear();
                            continue;
                        }
                        let log_event = LogEvent {
                            file: file_path.clone(),
                            line: line.trim().to_string(),
                        };

                        tx_clone.send(log_event).ok();
                        line.clear();
                    }
                }
            }

            _ => {}
        }
    }
}
