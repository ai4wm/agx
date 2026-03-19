pub mod detector;
pub mod process;
pub mod registry;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::PtySize;
use ratatui::style::Color;

use crate::agent::detector::{detect_state, AgentState};
use crate::agent::process::{AgentProcess, DEFAULT_PTY_SIZE};

#[derive(Clone, Debug)]
pub struct PaneSpec {
    pub label: String,
    pub command: String,
    pub detect_idle: Option<String>,
    pub accent_color: Option<Color>,
}

impl PaneSpec {
    pub fn new(label: String, command: String) -> Self {
        Self {
            label,
            command,
            detect_idle: None,
            accent_color: None,
        }
    }
}

pub struct AgentPane {
    pub label: String,
    pub parser: vt100::Parser,
    pub accent_color: Option<Color>,
    pub state: AgentState,
    process: AgentProcess,
    detect_idle: Option<String>,
}

impl AgentPane {
    #[allow(dead_code)]
    pub fn spawn_shell() -> Result<Self> {
        let shell = if cfg!(windows) {
            "powershell.exe"
        } else {
            "/bin/bash"
        };

        Self::spawn(PaneSpec::new(shell.to_string(), shell.to_string()))
    }

    pub fn spawn(spec: PaneSpec) -> Result<Self> {
        let argv = vec![spec.command.clone()];
        let process = AgentProcess::spawn(&argv, DEFAULT_PTY_SIZE)?;

        Ok(Self {
            label: spec.label,
            parser: vt100::Parser::new(DEFAULT_PTY_SIZE.rows, DEFAULT_PTY_SIZE.cols, 1_000),
            accent_color: spec.accent_color,
            state: AgentState::Working,
            process,
            detect_idle: spec.detect_idle,
        })
    }

    pub fn poll(&mut self) {
        for chunk in self.process.drain_output() {
            self.parser.process(&chunk);
        }

        let screen = self.parser.screen().contents();
        let exited = matches!(self.process.try_wait(), Ok(Some(_)));
        self.state = detect_state(&screen, self.detect_idle.as_deref(), exited);
    }

    pub fn send_key(&mut self, key: KeyEvent) {
        if self.is_dead() {
            return;
        }

        let bytes = encode_key(key);
        if bytes.is_empty() {
            return;
        }

        let _ = self.process.write_all(&bytes);
        self.state = AgentState::Working;
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        if rows == 0 || cols == 0 {
            return;
        }

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let _ = self.process.resize(size);
        self.parser.set_size(rows, cols);
    }

    pub fn is_dead(&self) -> bool {
        self.state == AgentState::Dead
    }

    pub fn is_idle(&self) -> bool {
        self.state == AgentState::Idle
    }
}

impl Drop for AgentPane {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

fn encode_key(key: KeyEvent) -> Vec<u8> {
    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if c.is_ascii() {
                    vec![(c.to_ascii_lowercase() as u8) & 0x1f]
                } else {
                    Vec::new()
                }
            } else {
                c.to_string().into_bytes()
            }
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        KeyCode::Insert => b"\x1b[2~".to_vec(),
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{encode_key, PaneSpec};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn encode_char_a() {
        assert_eq!(
            encode_key(key(KeyCode::Char('a'), KeyModifiers::empty())),
            b"a"
        );
    }

    #[test]
    fn encode_char_unicode() {
        assert_eq!(
            encode_key(key(KeyCode::Char('한'), KeyModifiers::empty())),
            "한".as_bytes().to_vec()
        );
    }

    #[test]
    fn encode_ctrl_c() {
        assert_eq!(
            encode_key(key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            vec![0x03]
        );
    }

    #[test]
    fn encode_ctrl_a() {
        assert_eq!(
            encode_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL)),
            vec![0x01]
        );
    }

    #[test]
    fn encode_ctrl_z() {
        assert_eq!(
            encode_key(key(KeyCode::Char('z'), KeyModifiers::CONTROL)),
            vec![0x1a]
        );
    }

    #[test]
    fn encode_enter() {
        assert_eq!(
            encode_key(key(KeyCode::Enter, KeyModifiers::empty())),
            vec![b'\r']
        );
    }

    #[test]
    fn encode_backspace() {
        assert_eq!(
            encode_key(key(KeyCode::Backspace, KeyModifiers::empty())),
            vec![0x7f]
        );
    }

    #[test]
    fn encode_tab_and_backtab() {
        assert_eq!(
            encode_key(key(KeyCode::Tab, KeyModifiers::empty())),
            vec![b'\t']
        );
        assert_eq!(
            encode_key(key(KeyCode::BackTab, KeyModifiers::empty())),
            b"\x1b[Z"
        );
    }

    #[test]
    fn encode_escape() {
        assert_eq!(
            encode_key(key(KeyCode::Esc, KeyModifiers::empty())),
            vec![0x1b]
        );
    }

    #[test]
    fn encode_arrow_keys() {
        assert_eq!(
            encode_key(key(KeyCode::Up, KeyModifiers::empty())),
            b"\x1b[A"
        );
        assert_eq!(
            encode_key(key(KeyCode::Down, KeyModifiers::empty())),
            b"\x1b[B"
        );
        assert_eq!(
            encode_key(key(KeyCode::Right, KeyModifiers::empty())),
            b"\x1b[C"
        );
        assert_eq!(
            encode_key(key(KeyCode::Left, KeyModifiers::empty())),
            b"\x1b[D"
        );
    }

    #[test]
    fn encode_home_end() {
        assert_eq!(
            encode_key(key(KeyCode::Home, KeyModifiers::empty())),
            b"\x1b[H"
        );
        assert_eq!(
            encode_key(key(KeyCode::End, KeyModifiers::empty())),
            b"\x1b[F"
        );
    }

    #[test]
    fn encode_delete_insert() {
        assert_eq!(
            encode_key(key(KeyCode::Delete, KeyModifiers::empty())),
            b"\x1b[3~"
        );
        assert_eq!(
            encode_key(key(KeyCode::Insert, KeyModifiers::empty())),
            b"\x1b[2~"
        );
    }

    #[test]
    fn encode_page_up_down() {
        assert_eq!(
            encode_key(key(KeyCode::PageUp, KeyModifiers::empty())),
            b"\x1b[5~"
        );
        assert_eq!(
            encode_key(key(KeyCode::PageDown, KeyModifiers::empty())),
            b"\x1b[6~"
        );
    }

    #[test]
    fn encode_unknown_returns_empty() {
        assert!(encode_key(key(KeyCode::F(12), KeyModifiers::empty())).is_empty());
    }

    #[test]
    fn pane_spec_new_defaults() {
        let spec = PaneSpec::new("test".to_string(), "echo".to_string());
        assert_eq!(spec.label, "test");
        assert_eq!(spec.command, "echo");
        assert!(spec.detect_idle.is_none());
        assert!(spec.accent_color.is_none());
    }
}
