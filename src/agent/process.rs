use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use anyhow::Result;
use portable_pty::{ExitStatus, PtySize};

use crate::terminal::pty::PtyProcess;

pub const DEFAULT_PTY_SIZE: PtySize = PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
};

pub struct AgentProcess {
    pty: PtyProcess,
    writer: Box<dyn Write + Send>,
    output_rx: Receiver<Vec<u8>>,
}

impl AgentProcess {
    pub fn spawn(command: &[String], size: PtySize) -> Result<Self> {
        let pty = PtyProcess::spawn(command, size)?;
        let mut reader = pty.try_clone_reader()?;
        let writer = pty.take_writer()?;
        let (tx, output_rx) = mpsc::channel();

        thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(read) => {
                        if tx.send(buf[..read].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            pty,
            writer,
            output_rx,
        })
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn try_read(&mut self) -> Option<Vec<u8>> {
        self.output_rx.try_recv().ok()
    }

    pub fn drain_output(&mut self) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        while let Some(chunk) = self.try_read() {
            chunks.push(chunk);
        }
        chunks
    }

    pub fn resize(&mut self, size: PtySize) -> Result<()> {
        self.pty.resize(size)
    }

    pub fn kill(&mut self) -> Result<()> {
        self.pty.kill()
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        self.pty.try_wait()
    }

    pub fn wait(&mut self) -> Result<ExitStatus> {
        self.pty.wait()
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{AgentProcess, DEFAULT_PTY_SIZE};

    #[test]
    fn spawn_and_kill() {
        let mut process =
            AgentProcess::spawn(&interactive_shell_command(), DEFAULT_PTY_SIZE).unwrap();
        process.kill().unwrap();

        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if process.try_wait().unwrap().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }

        panic!("process did not exit after kill");
    }

    #[test]
    fn eof_no_panic() {
        let mut process =
            AgentProcess::spawn(&interactive_shell_command(), DEFAULT_PTY_SIZE).unwrap();
        process.write_all(b"exit\r").unwrap();

        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if process.try_wait().unwrap().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }

        while process.try_read().is_some() {}
        assert!(process.try_read().is_none());
    }

    fn interactive_shell_command() -> Vec<String> {
        if cfg!(windows) {
            vec!["cmd.exe".to_string()]
        } else {
            vec!["/bin/sh".to_string()]
        }
    }
}
