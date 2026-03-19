use anyhow::{anyhow, Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

pub struct PtyProcess {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
}

impl PtyProcess {
    pub fn spawn(command: &[String], size: PtySize) -> Result<Self> {
        let binary = command
            .first()
            .ok_or_else(|| anyhow!("command cannot be empty"))?;

        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(size)
            .context("failed to create PTY pair")?;
        let portable_pty::PtyPair { master, slave } = pty_pair;

        let mut builder = CommandBuilder::new(binary);
        for arg in &command[1..] {
            builder.arg(arg);
        }

        builder.env("TERM", "xterm-256color");
        builder.env("TERM_PROGRAM", "agx");

        let child = slave
            .spawn_command(builder)
            .with_context(|| format!("failed to spawn command `{binary}`"))?;
        drop(slave);

        Ok(Self { master, child })
    }

    pub fn try_clone_reader(&self) -> Result<Box<dyn std::io::Read + Send>> {
        self.master
            .try_clone_reader()
            .context("failed to clone PTY master reader")
    }

    pub fn take_writer(&self) -> Result<Box<dyn std::io::Write + Send>> {
        self.master
            .take_writer()
            .context("failed to open PTY master writer")
    }

    #[allow(dead_code)]
    pub fn resize(&self, size: PtySize) -> Result<()> {
        self.master.resize(size).context("failed to resize PTY")
    }

    #[allow(dead_code)]
    pub fn wait(&mut self) -> Result<portable_pty::ExitStatus> {
        self.child.wait().context("failed to wait on child")
    }

    pub fn try_wait(&mut self) -> Result<Option<portable_pty::ExitStatus>> {
        self.child.try_wait().context("failed to poll child state")
    }

    #[allow(dead_code)]
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().context("failed to kill child")
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::thread;
    use std::time::{Duration, Instant};

    use portable_pty::PtySize;

    use super::PtyProcess;

    const DEFAULT_PTY_SIZE: PtySize = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    #[test]
    fn pty_write_read_roundtrip() {
        let mut process =
            PtyProcess::spawn(&interactive_shell_command(), DEFAULT_PTY_SIZE).unwrap();
        let reader = process.try_clone_reader().unwrap();
        let mut writer = process.take_writer().unwrap();

        writer.write_all(b"echo agx-pty-test\r").unwrap();
        writer.flush().unwrap();

        let output =
            wait_for_output(reader, "agx-pty-test", Duration::from_secs(5)).expect("pty output");
        assert!(output.contains("agx-pty-test"));

        let _ = process.kill();
    }

    #[test]
    fn pty_resize() {
        let process = PtyProcess::spawn(&interactive_shell_command(), DEFAULT_PTY_SIZE).unwrap();
        let size = PtySize {
            rows: 40,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        };

        assert!(process.resize(size).is_ok());
    }

    fn interactive_shell_command() -> Vec<String> {
        if cfg!(windows) {
            vec!["cmd.exe".to_string()]
        } else {
            vec!["/bin/sh".to_string()]
        }
    }

    fn wait_for_output(
        mut reader: Box<dyn Read + Send>,
        needle: &str,
        timeout: Duration,
    ) -> Option<String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let needle = needle.to_string();

        thread::spawn(move || {
            let start = Instant::now();
            let mut buffer = [0_u8; 4096];
            let mut output = String::new();

            while start.elapsed() < timeout {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        output.push_str(&String::from_utf8_lossy(&buffer[..read]));
                        if output.contains(&needle) {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }

            let _ = tx.send(output);
        });

        rx.recv_timeout(timeout).ok()
    }
}
