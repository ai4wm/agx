#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

pub fn temp_config(content: &str) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let agx_dir = dir.path().join("agx");
    fs::create_dir_all(&agx_dir).unwrap();
    let path = agx_dir.join("config.toml");
    fs::write(&path, content).unwrap();
    (dir, path)
}

pub fn echo_command(text: &str) -> Vec<String> {
    if cfg!(windows) {
        vec![
            "cmd.exe".to_string(),
            "/C".to_string(),
            "echo".to_string(),
            text.to_string(),
        ]
    } else {
        vec![
            "/bin/sh".to_string(),
            "-lc".to_string(),
            format!("echo {text}"),
        ]
    }
}

pub fn interactive_shell_command() -> Vec<String> {
    if cfg!(windows) {
        vec!["cmd.exe".to_string()]
    } else {
        vec!["/bin/sh".to_string()]
    }
}

pub fn wait_for_output<F>(mut next_chunk: F, needle: &str, timeout: Duration) -> Option<String>
where
    F: FnMut() -> Option<Vec<u8>>,
{
    let start = Instant::now();
    let mut output = String::new();

    while start.elapsed() < timeout {
        if let Some(chunk) = next_chunk() {
            output.push_str(&String::from_utf8_lossy(&chunk));
            if output.contains(needle) {
                return Some(output);
            }
        } else {
            thread::sleep(Duration::from_millis(25));
        }
    }

    None
}
