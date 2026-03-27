# agx 버그 수정 + 사이드바 기능 완성

## 프로젝트 경로
F:\1.Project\agx

## 버그 1: Alt+D (SplitRight) 누르면 앱이 종료됨

### 증상
- Alt+D 누르면 새 패인이 생기는 대신 앱이 종료됨
- Alt+S (SplitDown)도 동일할 가능성 있음
- 원인: split_current_workspace 또는 default_pane_spec에서 에러 발생 시 `?`로 전파되어 main까지 올라가서 앱 종료

### 수정 방법
1. src/app.rs의 handle_app_command에서 SplitRight, SplitDown의 에러를 전파하지 말고 무시 또는 상태바에 표시
2. 같은 패턴으로 NewSurface, NewWorkspace도 에러 전파 대신 무시 처리
3. 에러 원인을 찾기 위해 split_current_workspace에 eprintln 디버그 로그 추가

```rust
// 변경 전
AppCommand::SplitRight => self.split_current_workspace(SplitDirection::Vertical)?,
AppCommand::SplitDown => self.split_current_workspace(SplitDirection::Horizontal)?,
AppCommand::NewSurface => self.add_surface_to_focused_workspace()?,
AppCommand::NewWorkspace => self.create_workspace()?,

// 변경 후
AppCommand::SplitRight => { let _ = self.split_current_workspace(SplitDirection::Vertical); }
AppCommand::SplitDown => { let _ = self.split_current_workspace(SplitDirection::Horizontal); }
AppCommand::NewSurface => { let _ = self.add_surface_to_focused_workspace(); }
AppCommand::NewWorkspace => { let _ = self.create_workspace(); }
```

## 버그 2: 사이드바 포커스 시각 피드백 없음

### 증상
- Tab으로 사이드바에 포커스가 가도 화면에 변화 없음
- sidebar_focused, sidebar_cursor를 렌더링에서 사용하지 않음

### 수정: src/ui/sidebar.rs의 render 함수 전체 교체

```rust
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focused;
    let cursor = app.sidebar_cursor;
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };

    let mut lines = Vec::new();

    for (index, workspace) in app.workspaces.iter().enumerate() {
        let is_current = index == app.current_workspace;
        let is_cursor = focused && index == cursor;

        let (marker, name_style) = if is_cursor {
            (">", Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD))
        } else if is_current {
            ("*", Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD))
        } else {
            (" ", Style::default().fg(Color::White))
        };

        lines.push(Line::from(vec![Span::styled(
            format!("{marker} {}", workspace.name),
            name_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  panes:{}", workspace.panes.len()),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " no workspaces",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let title = if focused { " ↑↓:Move Enter:OK N:New D:Del " } else { " ws " };

    let sidebar = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(sidebar, area);
}
```

## 기능 추가: 사이드바에서 워크스페이스 추가/삭제

### 수정: src/app.rs의 handle_sidebar_key 메서드

```rust
fn handle_sidebar_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Up => {
            if self.sidebar_cursor > 0 {
                self.sidebar_cursor -= 1;
            }
        }
        KeyCode::Down => {
            if self.sidebar_cursor + 1 < self.workspaces.len() {
                self.sidebar_cursor += 1;
            }
        }
        KeyCode::Enter => {
            self.current_workspace = self
                .sidebar_cursor
                .min(self.workspaces.len().saturating_sub(1));
            self.sidebar_focused = false;
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            let _ = self.create_workspace();
            self.sidebar_cursor = self.workspaces.len().saturating_sub(1);
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if self.workspaces.len() > 1 {
                self.workspaces.remove(self.sidebar_cursor);
                if self.sidebar_cursor >= self.workspaces.len() {
                    self.sidebar_cursor = self.workspaces.len().saturating_sub(1);
                }
                if self.current_workspace >= self.workspaces.len() {
                    self.current_workspace = self.workspaces.len().saturating_sub(1);
                }
            }
        }
        KeyCode::Esc => {
            self.sidebar_focused = false;
        }
        _ => {}
    }
}
```

## 검증 기준
```powershell
cargo build
cargo test
cargo clippy --workspace -- -D warnings
cargo run -- --run powershell.exe --run powershell.exe
```

### 수동 테스트 체크리스트
- [ ] Alt+D: 새 패인 생성 (앱 종료 안 됨)
- [ ] Alt+S: 아래로 분할
- [ ] Tab: 사이드바 → Pane1 → Pane2 → 사이드바 순환
- [ ] 사이드바 포커스 시: 보더 Cyan, 제목 변경, 커서 반전 하이라이트
- [ ] 사이드바에서 ↑↓: 워크스페이스 이동
- [ ] 사이드바에서 Enter: 워크스페이스 전환 + 패인 복귀
- [ ] 사이드바에서 N: 새 워크스페이스 추가
- [ ] 사이드바에서 D: 워크스페이스 삭제 (최소 1개 유지)
- [ ] 사이드바에서 Esc: 패인 복귀
- [ ] Alt+Q: 정상 종료

## 제약사항
- 기존 테스트 깨지면 안 됨
- 크로스플랫폼 유지
- 에러 처리: 앱 종료 대신 무시 또는 상태바 표시

---

## 현재 src/ 전체 파일 내용
아래는 현재 워크스페이스의 `src/` 전체 파일 전문입니다. 특히 `src/app.rs`, `src/ui/sidebar.rs`, `src/terminal/input.rs`는 우선 참고 대상입니다.

## src/agent/detector.rs
`ust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentState {
    Idle,
    Working,
    Dead,
}

pub fn detect_state(output: &str, idle_pattern: Option<&str>, exited: bool) -> AgentState {
    if exited {
        return AgentState::Dead;
    }

    match idle_pattern {
        Some("") => AgentState::Idle,
        Some(pattern) if output.contains(pattern) => AgentState::Idle,
        _ => AgentState::Working,
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_state, AgentState};

    #[test]
    fn detect_idle_by_prompt() {
        assert_eq!(
            detect_state("user@host:~$ ", Some("$ "), false),
            AgentState::Idle
        );
    }

    #[test]
    fn detect_working_no_match() {
        assert_eq!(
            detect_state("Thinking...", Some("$ "), false),
            AgentState::Working
        );
    }

    #[test]
    fn detect_dead_on_empty() {
        assert_eq!(detect_state("", Some("$ "), true), AgentState::Dead);
    }

    #[test]
    fn detect_custom_pattern() {
        assert_eq!(detect_state(" ", Some(""), false), AgentState::Idle);
    }
}

```

## src/agent/mod.rs
`ust
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

```

## src/agent/process.rs
`ust
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

```

## src/agent/registry.rs
`ust
use std::collections::BTreeMap;

use anyhow::{bail, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentDefinition {
    pub name: String,
    pub command: String,
    pub detect_idle: Option<String>,
    pub color: Option<String>,
}

#[derive(Default)]
pub struct AgentRegistry {
    agents: BTreeMap<String, AgentDefinition>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, definition: AgentDefinition) -> Result<()> {
        if self.agents.contains_key(&definition.name) {
            bail!("duplicate agent registration for `{}`", definition.name);
        }

        self.agents.insert(definition.name.clone(), definition);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&AgentDefinition> {
        self.agents.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentDefinition, AgentRegistry};

    fn sample_agent() -> AgentDefinition {
        AgentDefinition {
            name: "claude".to_string(),
            command: "claude".to_string(),
            detect_idle: None,
            color: Some("cyan".to_string()),
        }
    }

    #[test]
    fn register_and_lookup() {
        let mut registry = AgentRegistry::new();
        registry.register(sample_agent()).unwrap();
        assert!(registry.get("claude").is_some());
    }

    #[test]
    fn lookup_missing() {
        let registry = AgentRegistry::new();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn reject_duplicate() {
        let mut registry = AgentRegistry::new();
        registry.register(sample_agent()).unwrap();
        assert!(registry.register(sample_agent()).is_err());
    }
}

```

## src/app.rs
`ust
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::config::loader::Config;
use crate::terminal::input::{command_for_key, AppCommand};
use crate::ui::{self, layout::LayoutState};
use crate::workspace::Workspace;
use crate::SplitDirection;

#[derive(Clone, Debug, Default)]
pub struct AppOptions {
    pub run: Vec<String>,
    pub split: Option<SplitDirection>,
}

pub struct App {
    pub workspaces: Vec<Workspace>,
    pub current_workspace: usize,
    pub should_quit: bool,
    pub show_sidebar: bool,
    pub sidebar_focused: bool,
    pub sidebar_cursor: usize,
    config: Config,
    next_workspace_id: usize,
    default_split: SplitDirection,
}

impl App {
    pub fn new(options: AppOptions) -> Result<Self> {
        let config = Config::load()?;
        let default_split = options.split.unwrap_or(config.default_split()?);
        let mut specs = Vec::new();

        if options.run.is_empty() {
            specs.push(config.default_pane_spec()?);
        } else {
            for command in options.run {
                specs.push(config.resolve_pane_spec(&command)?);
            }
        }

        let mut specs = specs.into_iter();
        let first_spec = specs.next().expect("initial pane spec");
        let mut workspace = Workspace::new("ws1", first_spec, default_split)?;

        for spec in specs {
            match default_split {
                SplitDirection::Vertical => workspace.split_right(spec)?,
                SplitDirection::Horizontal => workspace.split_down(spec)?,
            }
        }
        workspace.focused_pane = 0;

        Ok(Self {
            workspaces: vec![workspace],
            current_workspace: 0,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: false,
            sidebar_cursor: 0,
            config,
            next_workspace_id: 2,
            default_split,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let result = self.main_loop(&mut terminal).await;
        ratatui::restore();
        result
    }

    async fn main_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            if self.should_quit {
                break;
            }

            for workspace in &mut self.workspaces {
                workspace.poll();
            }

            let (pane_surface_counts, split) = self
                .current_workspace()
                .map(|workspace| (workspace.pane_surface_counts(), workspace.split))
                .unwrap_or_else(|| (Vec::new(), self.default_split));

            terminal.draw(|frame| {
                let layout = ui::layout::compute_layout(
                    frame.area(),
                    self.show_sidebar,
                    &pane_surface_counts,
                    split,
                );
                self.resize_current_workspace(&layout);
                ui::render(frame, self, &layout);
            })?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn resize_current_workspace(&mut self, layout: &LayoutState) {
        let Some(workspace) = self.current_workspace_mut() else {
            return;
        };

        for (pane, pane_layout) in workspace.panes.iter_mut().zip(&layout.pane_layouts) {
            pane.resize(
                pane_layout.content.height.max(1),
                pane_layout.content.width.max(1),
            );
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(command) = command_for_key(key) {
            self.handle_app_command(command)?;
            return Ok(());
        }

        if self.sidebar_focused {
            self.handle_sidebar_key(key);
            return Ok(());
        }

        if let Some(workspace) = self.current_workspace_mut() {
            workspace.send_key_to_focused(key);
        }

        Ok(())
    }

    fn handle_app_command(&mut self, command: AppCommand) -> Result<()> {
        match command {
            AppCommand::SplitRight => self.split_current_workspace(SplitDirection::Vertical)?,
            AppCommand::SplitDown => self.split_current_workspace(SplitDirection::Horizontal)?,
            AppCommand::NewSurface => self.add_surface_to_focused_workspace()?,
            AppCommand::CloseSurface => self.close_current_surface(),
            AppCommand::FocusLeft => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.focus_left();
                }
            }
            AppCommand::FocusRight => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.focus_right();
                }
            }
            AppCommand::FocusUp => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.focus_up();
                }
            }
            AppCommand::FocusDown => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.focus_down();
                }
            }
            AppCommand::PrevSurface => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.prev_surface();
                }
            }
            AppCommand::NextSurface => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.next_surface();
                }
            }
            AppCommand::ToggleSidebar => self.toggle_sidebar(),
            AppCommand::NewWorkspace => self.create_workspace()?,
            AppCommand::CloseWorkspace => self.close_current_workspace(),
            AppCommand::SwitchWorkspace(index) => {
                if index < self.workspaces.len() {
                    self.current_workspace = index;
                    if self.sidebar_focused {
                        self.sidebar_cursor = index;
                    }
                }
            }
            AppCommand::CycleFocusForward => self.cycle_focus(true),
            AppCommand::CycleFocusBackward => self.cycle_focus(false),
            AppCommand::Quit => self.should_quit = true,
        }

        Ok(())
    }

    fn split_current_workspace(&mut self, split: SplitDirection) -> Result<()> {
        let spec = self.config.default_pane_spec()?;
        if let Some(workspace) = self.current_workspace_mut() {
            match split {
                SplitDirection::Vertical => workspace.split_right(spec)?,
                SplitDirection::Horizontal => workspace.split_down(spec)?,
            }
        } else {
            self.create_workspace()?;
        }
        Ok(())
    }

    fn add_surface_to_focused_workspace(&mut self) -> Result<()> {
        let spec = self.config.default_pane_spec()?;
        if let Some(workspace) = self.current_workspace_mut() {
            workspace.add_surface_to_focused(spec)?;
        } else {
            self.create_workspace()?;
        }
        Ok(())
    }

    fn create_workspace(&mut self) -> Result<()> {
        let name = format!("ws{}", self.next_workspace_id);
        self.next_workspace_id += 1;

        let spec = self.config.default_pane_spec()?;
        let workspace = Workspace::new(name, spec, self.default_split)?;
        self.workspaces.push(workspace);
        self.current_workspace = self.workspaces.len().saturating_sub(1);
        if self.sidebar_focused {
            self.sidebar_cursor = self.current_workspace;
        }
        Ok(())
    }

    fn toggle_sidebar(&mut self) {
        if self.show_sidebar {
            self.show_sidebar = false;
            self.sidebar_focused = false;
            return;
        }

        self.show_sidebar = true;
        self.sidebar_focused = true;
        self.sidebar_cursor = self.current_workspace;
    }

    fn cycle_focus(&mut self, forward: bool) {
        let pane_count = self
            .current_workspace()
            .map(|ws| ws.panes.len())
            .unwrap_or(0);
        if pane_count == 0 {
            return;
        }

        let total = if self.show_sidebar {
            pane_count + 1
        } else {
            pane_count
        };

        let current = if self.show_sidebar {
            if self.sidebar_focused {
                0
            } else {
                1 + self
                    .current_workspace()
                    .map(|ws| ws.focused_pane)
                    .unwrap_or(0)
            }
        } else {
            self.current_workspace()
                .map(|ws| ws.focused_pane)
                .unwrap_or(0)
        };

        let next = if forward {
            (current + 1) % total
        } else {
            (current + total - 1) % total
        };

        if self.show_sidebar && next == 0 {
            self.sidebar_focused = true;
            self.sidebar_cursor = self.current_workspace;
            return;
        }

        self.sidebar_focused = false;
        let pane_index = if self.show_sidebar { next - 1 } else { next };
        if let Some(workspace) = self.current_workspace_mut() {
            workspace.focused_pane = pane_index.min(workspace.panes.len().saturating_sub(1));
        }
    }

    fn handle_sidebar_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => {
                if self.sidebar_cursor > 0 {
                    self.sidebar_cursor -= 1;
                }
            }
            KeyCode::Down => {
                if self.sidebar_cursor + 1 < self.workspaces.len() {
                    self.sidebar_cursor += 1;
                }
            }
            KeyCode::Enter => {
                self.current_workspace = self
                    .sidebar_cursor
                    .min(self.workspaces.len().saturating_sub(1));
                self.sidebar_focused = false;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                let _ = self.create_workspace();
                self.sidebar_cursor = self.workspaces.len().saturating_sub(1);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.delete_sidebar_workspace();
            }
            KeyCode::Esc => {
                self.sidebar_focused = false;
            }
            _ => {}
        }
    }

    fn delete_sidebar_workspace(&mut self) {
        if self.workspaces.len() <= 1 {
            return;
        }

        let remove_index = self
            .sidebar_cursor
            .min(self.workspaces.len().saturating_sub(1));
        self.workspaces.remove(remove_index);

        if self.sidebar_cursor >= self.workspaces.len() {
            self.sidebar_cursor = self.workspaces.len().saturating_sub(1);
        }

        if self.current_workspace == remove_index {
            self.current_workspace = remove_index.min(self.workspaces.len().saturating_sub(1));
        } else if self.current_workspace > remove_index {
            self.current_workspace -= 1;
        }
    }

    fn close_current_surface(&mut self) {
        let Some(workspace) = self.current_workspace_mut() else {
            return;
        };

        workspace.close_current_surface();
        self.prune_empty_workspace();
    }

    pub fn current_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.current_workspace)
    }

    fn current_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspaces.get_mut(self.current_workspace)
    }

    fn close_current_workspace(&mut self) {
        if self.workspaces.is_empty() {
            return;
        }

        self.workspaces.remove(self.current_workspace);
        if self.current_workspace >= self.workspaces.len() && !self.workspaces.is_empty() {
            self.current_workspace = self.workspaces.len() - 1;
        } else if self.workspaces.is_empty() {
            self.current_workspace = 0;
            self.sidebar_cursor = 0;
            self.sidebar_focused = false;
            return;
        }

        if self.sidebar_cursor >= self.workspaces.len() {
            self.sidebar_cursor = self.current_workspace;
        }
    }

    fn prune_empty_workspace(&mut self) {
        if self.current_workspace().is_some_and(Workspace::is_empty) {
            self.close_current_workspace();
        }
    }

    pub fn workspaces_empty(&self) -> bool {
        self.workspaces.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{App, AppOptions};
    use crate::agent::PaneSpec;
    use crate::config::loader::Config;
    use crate::terminal::input::AppCommand;
    use crate::workspace::Workspace;
    use crate::SplitDirection;

    #[test]
    fn alt_b_toggles_sidebar() {
        let mut app = empty_app();
        app.handle_key(alt_key(KeyCode::Char('b'))).unwrap();
        assert!(!app.show_sidebar);
    }

    #[test]
    fn alt_q_sets_should_quit() {
        let mut app = empty_app();
        app.handle_key(alt_key(KeyCode::Char('q'))).unwrap();
        assert!(app.should_quit);
    }

    #[test]
    fn alt_digit_switches_workspace() {
        let mut app = App {
            workspaces: vec![empty_workspace("ws1"), empty_workspace("ws2")],
            current_workspace: 0,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: false,
            sidebar_cursor: 0,
            config: Config::default(),
            next_workspace_id: 3,
            default_split: SplitDirection::Vertical,
        };

        app.handle_key(alt_key(KeyCode::Char('2'))).unwrap();
        assert_eq!(app.current_workspace, 1);
    }

    #[test]
    fn close_workspace_retargets_selection() {
        let mut app = App {
            workspaces: vec![empty_workspace("ws1"), empty_workspace("ws2")],
            current_workspace: 1,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: false,
            sidebar_cursor: 0,
            config: Config::default(),
            next_workspace_id: 3,
            default_split: SplitDirection::Vertical,
        };

        app.handle_app_command(AppCommand::CloseWorkspace).unwrap();
        assert_eq!(app.workspaces.len(), 1);
        assert_eq!(app.current_workspace, 0);
    }

    #[test]
    fn app_options_default_is_empty() {
        let options = AppOptions::default();
        assert!(options.run.is_empty());
        assert_eq!(options.split, None);
    }

    #[test]
    fn tab_cycles_focus_forward_across_sidebar_and_panes() {
        let mut app = app_with_workspace(workspace_with_panes(2));
        app.handle_key(tab_key()).unwrap();
        assert_eq!(app.current_workspace().unwrap().focused_pane, 1);
        assert!(!app.sidebar_focused);

        app.handle_key(tab_key()).unwrap();
        assert!(app.sidebar_focused);
        assert_eq!(app.sidebar_cursor, 0);

        app.handle_key(tab_key()).unwrap();
        assert!(!app.sidebar_focused);
        assert_eq!(app.current_workspace().unwrap().focused_pane, 0);
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        let mut app = app_with_workspace(workspace_with_panes(2));
        app.sidebar_focused = true;
        app.sidebar_cursor = 0;

        app.handle_key(shift_tab_key()).unwrap();
        assert!(!app.sidebar_focused);
        assert_eq!(app.current_workspace().unwrap().focused_pane, 1);
    }

    #[test]
    fn alt_b_opens_sidebar_and_focuses_current_workspace() {
        let mut app = app_with_workspace(workspace_with_panes(1));
        app.show_sidebar = false;

        app.handle_key(alt_key(KeyCode::Char('b'))).unwrap();
        assert!(app.show_sidebar);
        assert!(app.sidebar_focused);
        assert_eq!(app.sidebar_cursor, 0);
    }

    #[test]
    fn sidebar_enter_switches_workspace() {
        let mut app = App {
            workspaces: vec![workspace_with_panes(1), workspace_with_panes(1)],
            current_workspace: 0,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: true,
            sidebar_cursor: 1,
            config: Config::default(),
            next_workspace_id: 3,
            default_split: SplitDirection::Vertical,
        };

        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()))
            .unwrap();
        assert_eq!(app.current_workspace, 1);
        assert!(!app.sidebar_focused);
    }

    #[test]
    fn sidebar_n_creates_workspace_and_moves_cursor() {
        let mut app = app_with_workspace(workspace_with_panes(1));
        app.sidebar_focused = true;
        app.sidebar_cursor = 0;

        app.handle_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty()))
            .unwrap();
        assert_eq!(app.workspaces.len(), 2);
        assert_eq!(app.sidebar_cursor, 1);
        assert_eq!(app.current_workspace, 1);
    }

    #[test]
    fn sidebar_d_deletes_cursor_workspace_and_retargets_current() {
        let mut app = App {
            workspaces: vec![
                named_workspace("ws1"),
                named_workspace("ws2"),
                named_workspace("ws3"),
            ],
            current_workspace: 2,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: true,
            sidebar_cursor: 1,
            config: Config::default(),
            next_workspace_id: 4,
            default_split: SplitDirection::Vertical,
        };

        app.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()))
            .unwrap();
        assert_eq!(app.workspaces.len(), 2);
        assert_eq!(app.current_workspace, 1);
        assert_eq!(app.sidebar_cursor, 1);
        assert_eq!(app.workspaces[1].name, "ws3");
    }

    fn empty_app() -> App {
        App {
            workspaces: vec![empty_workspace("ws1")],
            current_workspace: 0,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: false,
            sidebar_cursor: 0,
            config: Config::default(),
            next_workspace_id: 2,
            default_split: SplitDirection::Vertical,
        }
    }

    fn app_with_workspace(workspace: Workspace) -> App {
        App {
            workspaces: vec![workspace],
            current_workspace: 0,
            should_quit: false,
            show_sidebar: true,
            sidebar_focused: false,
            sidebar_cursor: 0,
            config: Config::default(),
            next_workspace_id: 2,
            default_split: SplitDirection::Vertical,
        }
    }

    fn empty_workspace(name: &str) -> Workspace {
        Workspace {
            name: name.to_string(),
            panes: Vec::new(),
            focused_pane: 0,
            split: SplitDirection::Vertical,
        }
    }

    fn named_workspace(name: &str) -> Workspace {
        let mut workspace = workspace_with_panes(1);
        workspace.name = name.to_string();
        workspace
    }

    fn workspace_with_panes(count: usize) -> Workspace {
        let mut workspace = Workspace::new(
            "ws1",
            interactive_shell_spec("surface-1"),
            SplitDirection::Vertical,
        )
        .unwrap();
        for index in 1..count {
            workspace
                .split_right(interactive_shell_spec(&format!("surface-{}", index + 1)))
                .unwrap();
        }
        workspace.focused_pane = 0;
        workspace
    }

    fn alt_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
    }

    fn tab_key() -> KeyEvent {
        KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())
    }

    fn shift_tab_key() -> KeyEvent {
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)
    }

    fn interactive_shell_spec(label: &str) -> PaneSpec {
        let command = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };
        PaneSpec::new(label.to_string(), command.to_string())
    }
}

```

## src/config.rs
`ust
pub mod loader;

pub use loader::{AgentConfig, Config, DefaultsConfig, KeybindConfig};

```

## src/config/loader.rs
`ust
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use ratatui::style::Color;
use serde::Deserialize;

use crate::agent::registry::AgentDefinition;
use crate::agent::PaneSpec;
use crate::SplitDirection;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keybind: KeybindConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub agent: Vec<AgentConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct KeybindConfig {}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DefaultsConfig {
    pub shell: Option<String>,
    pub split: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub detect_idle: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::path()?)
    }

    pub fn path() -> Result<PathBuf> {
        let base =
            dirs::config_dir().ok_or_else(|| anyhow!("failed to resolve config directory"))?;
        Ok(base.join("agx").join("config.toml"))
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read config file at {}", path.display()))?;
        Self::load_from_str(&contents)
    }

    pub fn load_from_str(contents: &str) -> Result<Self> {
        if contents.trim().is_empty() {
            return Ok(Self::default());
        }

        let config: Self = toml::from_str(contents).context("failed to parse config contents")?;
        Ok(config)
    }

    pub fn default_split(&self) -> Result<SplitDirection> {
        match self.defaults.split.as_deref() {
            Some(value) => SplitDirection::from_config_value(value)
                .ok_or_else(|| anyhow!("invalid defaults.split value `{value}`")),
            None => Ok(SplitDirection::default()),
        }
    }

    pub fn default_pane_spec(&self) -> Result<PaneSpec> {
        let shell = self.defaults.shell.clone().unwrap_or_else(default_shell);
        self.resolve_pane_spec(&shell)
    }

    pub fn resolve_pane_spec(&self, spec: &str) -> Result<PaneSpec> {
        if let Some(agent) = self.agent.iter().find(|candidate| candidate.name == spec) {
            let accent_color = match agent.color.as_deref() {
                Some(value) => Some(parse_color(value)?),
                None => None,
            };

            return Ok(PaneSpec {
                label: agent.name.clone(),
                command: agent.command.clone(),
                detect_idle: normalize_pattern(agent.detect_idle.clone()),
                accent_color,
            });
        }

        Ok(PaneSpec::new(spec.to_string(), spec.to_string()))
    }

    pub fn agent_definitions(&self) -> Vec<AgentDefinition> {
        self.agent
            .iter()
            .map(|agent| AgentDefinition {
                name: agent.name.clone(),
                command: agent.command.clone(),
                detect_idle: normalize_pattern(agent.detect_idle.clone()),
                color: agent.color.clone(),
            })
            .collect()
    }
}

fn default_shell() -> String {
    if cfg!(windows) {
        "powershell.exe".to_string()
    } else {
        "/bin/bash".to_string()
    }
}

fn normalize_pattern(pattern: Option<String>) -> Option<String> {
    pattern.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn parse_color(value: &str) -> Result<Color> {
    let lower = value.trim().to_ascii_lowercase();
    let color = match lower.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "dark-gray" | "dark_grey" | "darkgrey" => Color::DarkGray,
        value if value.starts_with('#') && value.len() == 7 => {
            let r = u8::from_str_radix(&value[1..3], 16)
                .with_context(|| format!("invalid hex color `{value}`"))?;
            let g = u8::from_str_radix(&value[3..5], 16)
                .with_context(|| format!("invalid hex color `{value}`"))?;
            let b = u8::from_str_radix(&value[5..7], 16)
                .with_context(|| format!("invalid hex color `{value}`"))?;
            Color::Rgb(r, g, b)
        }
        _ => return Err(anyhow!("unsupported color `{value}`")),
    };

    Ok(color)
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::Config;
    use crate::SplitDirection;

    #[test]
    fn parse_valid_config() {
        let config = Config::load_from_str(
            r#"
[defaults]
shell = "powershell.exe"
split = "horizontal"

[[agent]]
name = "claude"
command = "claude"
detect_idle = ""
color = "cyan"

[[agent]]
name = "codex"
command = "codex"
detect_idle = ">"
color = "green"
"#,
        )
        .unwrap();

        assert_eq!(config.agent.len(), 2);
        assert_eq!(config.default_split().unwrap(), SplitDirection::Horizontal);
    }

    #[test]
    fn empty_file_uses_defaults() {
        let config = Config::load_from_str("").unwrap();

        assert_eq!(config.default_split().unwrap(), SplitDirection::Vertical);
        assert!(!config.default_pane_spec().unwrap().command.is_empty());
    }

    #[test]
    fn malformed_toml_returns_error() {
        assert!(Config::load_from_str("[[[broken").is_err());
    }

    #[test]
    fn parse_color_named() {
        assert_eq!(super::parse_color("cyan").unwrap(), Color::Cyan);
        assert_eq!(super::parse_color("Red").unwrap(), Color::Red);
        assert_eq!(super::parse_color("BLUE").unwrap(), Color::Blue);
    }

    #[test]
    fn parse_color_hex() {
        assert_eq!(
            super::parse_color("#ff00aa").unwrap(),
            Color::Rgb(255, 0, 170)
        );
        assert_eq!(super::parse_color("#000000").unwrap(), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn parse_color_invalid() {
        assert!(super::parse_color("rainbow").is_err());
        assert!(super::parse_color("#zzzzzz").is_err());
        assert!(super::parse_color("#fff").is_err());
    }

    #[test]
    fn resolve_pane_spec_registered_agent() {
        let config = Config::load_from_str(
            r#"
[[agent]]
name = "claude"
command = "claude-code"
detect_idle = ""
color = "cyan"
"#,
        )
        .unwrap();

        let spec = config.resolve_pane_spec("claude").unwrap();
        assert_eq!(spec.label, "claude");
        assert_eq!(spec.command, "claude-code");
        assert_eq!(spec.detect_idle, None);
        assert_eq!(spec.accent_color, Some(Color::Cyan));
    }

    #[test]
    fn resolve_pane_spec_raw_command() {
        let config = Config::default();
        let spec = config.resolve_pane_spec("my-tool").unwrap();
        assert_eq!(spec.label, "my-tool");
        assert_eq!(spec.command, "my-tool");
        assert!(spec.detect_idle.is_none());
        assert!(spec.accent_color.is_none());
    }

    #[test]
    fn default_split_invalid() {
        let config = Config::load_from_str(
            r#"
[defaults]
split = "diagonal"
"#,
        )
        .unwrap();

        assert!(config.default_split().is_err());
    }

    #[test]
    fn normalize_pattern_trims() {
        assert_eq!(
            super::normalize_pattern(Some("  ready  ".to_string())),
            Some("ready".to_string())
        );
    }

    #[test]
    fn normalize_pattern_empty_is_none() {
        assert_eq!(super::normalize_pattern(Some("   ".to_string())), None);
        assert_eq!(super::normalize_pattern(None), None);
    }

    #[test]
    fn legacy_keybind_section_is_ignored() {
        let config = Config::load_from_str(
            r#"
[keybind]
prefix = "Ctrl-a"

[defaults]
split = "vertical"
"#,
        )
        .unwrap();

        assert_eq!(config.default_split().unwrap(), SplitDirection::Vertical);
    }
}

```

## src/lib.rs
`ust
pub mod agent;
pub mod app;
pub mod config;
pub mod pane;
pub mod surface;
pub mod terminal;
pub mod ui;
pub mod workspace;

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum SplitDirection {
    #[default]
    Vertical,
    Horizontal,
}

impl SplitDirection {
    pub fn from_config_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "vertical" => Some(Self::Vertical),
            "horizontal" => Some(Self::Horizontal),
            _ => None,
        }
    }
}

```

## src/main.rs
`ust
use agx::app::{App, AppOptions};
use agx::SplitDirection;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "agx",
    version,
    about = "Cross-platform AI agent terminal multiplexer"
)]
struct Cli {
    /// Agent commands to run. Repeat the flag to launch multiple panes.
    #[arg(short, long)]
    run: Vec<String>,

    /// Split direction for the pane layout. Overrides config.toml.
    #[arg(short, long, value_enum)]
    split: Option<SplitDirection>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async move {
        let mut app = App::new(AppOptions {
            run: cli.run,
            split: cli.split,
        })?;
        app.run().await
    })
}

```

## src/pane.rs
`ust
use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::agent::PaneSpec;
use crate::surface::Surface;

/// Pane = a split region that owns one or more surfaces.
pub struct Pane {
    pub surfaces: Vec<Surface>,
    pub current_surface: usize,
}

impl Pane {
    pub fn new(spec: PaneSpec) -> Result<Self> {
        Ok(Self {
            surfaces: vec![Surface::new(spec)?],
            current_surface: 0,
        })
    }

    pub fn add_surface(&mut self, spec: PaneSpec) -> Result<()> {
        self.surfaces.push(Surface::new(spec)?);
        self.current_surface = self.surfaces.len().saturating_sub(1);
        Ok(())
    }

    pub fn close_current_surface(&mut self) {
        if self.surfaces.is_empty() {
            return;
        }

        self.surfaces.remove(self.current_surface);
        if self.current_surface >= self.surfaces.len() && !self.surfaces.is_empty() {
            self.current_surface = self.surfaces.len() - 1;
        } else if self.surfaces.is_empty() {
            self.current_surface = 0;
        }
    }

    pub fn next_surface(&mut self) {
        if self.current_surface + 1 < self.surfaces.len() {
            self.current_surface += 1;
        }
    }

    pub fn prev_surface(&mut self) {
        if self.current_surface > 0 {
            self.current_surface -= 1;
        }
    }

    pub fn current_surface(&self) -> Option<&Surface> {
        self.surfaces.get(self.current_surface)
    }

    pub fn current_surface_mut(&mut self) -> Option<&mut Surface> {
        self.surfaces.get_mut(self.current_surface)
    }

    pub fn poll(&mut self) {
        for surface in &mut self.surfaces {
            surface.poll();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    pub fn send_key(&mut self, key: KeyEvent) {
        if let Some(surface) = self.current_surface_mut() {
            surface.agent.send_key(key);
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        for surface in &mut self.surfaces {
            surface.agent.resize(rows, cols);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Pane;
    use crate::agent::PaneSpec;

    #[test]
    fn close_current_surface_on_empty_is_noop() {
        let mut pane = Pane {
            surfaces: Vec::new(),
            current_surface: 0,
        };
        pane.close_current_surface();
        assert!(pane.is_empty());
        assert_eq!(pane.current_surface, 0);
    }

    #[test]
    fn prev_surface_at_zero_stays() {
        let mut pane = Pane {
            surfaces: Vec::new(),
            current_surface: 0,
        };
        pane.prev_surface();
        assert_eq!(pane.current_surface, 0);
    }

    #[test]
    fn next_surface_at_end_stays() {
        let mut pane = pane_with_surfaces(2);
        pane.current_surface = 1;
        pane.next_surface();
        assert_eq!(pane.current_surface, 1);
    }

    #[test]
    fn add_surface_selects_new_surface() {
        let mut pane = Pane::new(interactive_shell_spec("surface-1")).unwrap();
        pane.add_surface(interactive_shell_spec("surface-2"))
            .unwrap();
        assert_eq!(pane.surfaces.len(), 2);
        assert_eq!(pane.current_surface, 1);
        assert_eq!(pane.current_surface().unwrap().label, "surface-2");
    }

    #[test]
    fn close_current_surface_retargets_selection() {
        let mut pane = pane_with_surfaces(3);
        pane.current_surface = 1;
        pane.close_current_surface();
        assert_eq!(pane.surfaces.len(), 2);
        assert_eq!(pane.current_surface, 1);
    }

    #[test]
    fn prev_surface_decrements() {
        let mut pane = pane_with_surfaces(2);
        pane.current_surface = 1;
        pane.prev_surface();
        assert_eq!(pane.current_surface, 0);
    }

    #[test]
    fn pane_is_empty_tracks_surfaces() {
        let pane = Pane {
            surfaces: Vec::new(),
            current_surface: 0,
        };
        assert!(pane.is_empty());
    }

    fn pane_with_surfaces(count: usize) -> Pane {
        let mut pane = Pane::new(interactive_shell_spec("surface-1")).unwrap();
        for index in 1..count {
            pane.add_surface(interactive_shell_spec(&format!("surface-{}", index + 1)))
                .unwrap();
        }
        pane
    }

    fn interactive_shell_spec(label: &str) -> PaneSpec {
        let command = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };
        PaneSpec::new(label.to_string(), command.to_string())
    }
}

```

## src/surface.rs
`ust
use anyhow::Result;

use crate::agent::{AgentPane, PaneSpec};

/// Surface = a tab inside a pane.
pub struct Surface {
    pub label: String,
    pub agent: AgentPane,
}

impl Surface {
    pub fn new(spec: PaneSpec) -> Result<Self> {
        let label = spec.label.clone();
        let agent = AgentPane::spawn(spec)?;
        Ok(Self { label, agent })
    }

    pub fn poll(&mut self) {
        self.agent.poll();
    }

    pub fn is_exited(&self) -> bool {
        self.agent.is_dead()
    }
}

#[cfg(test)]
mod tests {
    use super::Surface;
    use crate::agent::PaneSpec;

    #[test]
    #[ignore = "spawns a PTY process"]
    fn surface_new_spawns_agent() {
        let surface = Surface::new(interactive_shell_spec("surface-1")).unwrap();
        assert_eq!(surface.label, "surface-1");
    }

    fn interactive_shell_spec(label: &str) -> PaneSpec {
        let command = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };
        PaneSpec::new(label.to_string(), command.to_string())
    }
}

```

## src/terminal/input.rs
`ust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppCommand {
    SplitRight,
    SplitDown,
    NewSurface,
    CloseSurface,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    PrevSurface,
    NextSurface,
    ToggleSidebar,
    NewWorkspace,
    CloseWorkspace,
    SwitchWorkspace(usize),
    CycleFocusForward,
    CycleFocusBackward,
    Quit,
}

pub fn command_for_key(key: KeyEvent) -> Option<AppCommand> {
    if key.code == KeyCode::BackTab && !key.modifiers.contains(KeyModifiers::ALT) {
        return Some(AppCommand::CycleFocusBackward);
    }

    // Tab / Shift+Tab: focus cycle without Alt.
    if key.code == KeyCode::Tab && !key.modifiers.contains(KeyModifiers::ALT) {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            return Some(AppCommand::CycleFocusBackward);
        }
        return Some(AppCommand::CycleFocusForward);
    }

    if !key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }

    match key.code {
        KeyCode::Left => Some(AppCommand::FocusLeft),
        KeyCode::Right => Some(AppCommand::FocusRight),
        KeyCode::Up => Some(AppCommand::FocusUp),
        KeyCode::Down => Some(AppCommand::FocusDown),
        KeyCode::Char('[') => Some(AppCommand::PrevSurface),
        KeyCode::Char(']') => Some(AppCommand::NextSurface),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'d') => Some(AppCommand::SplitRight),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'s') => Some(AppCommand::SplitDown),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'t') => Some(AppCommand::NewSurface),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'w') => Some(AppCommand::CloseSurface),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'c') => Some(AppCommand::NewWorkspace),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'x') => Some(AppCommand::CloseWorkspace),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'b') => Some(AppCommand::ToggleSidebar),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&'q') => Some(AppCommand::Quit),
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            Some(AppCommand::SwitchWorkspace(c as usize - '1' as usize))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{command_for_key, AppCommand};

    #[test]
    fn tab_cycles_focus_forward() {
        assert_eq!(
            command_for_key(key(KeyCode::Tab, KeyModifiers::empty())),
            Some(AppCommand::CycleFocusForward)
        );
    }

    #[test]
    fn shift_tab_cycles_focus_backward() {
        assert_eq!(
            command_for_key(key(KeyCode::Tab, KeyModifiers::SHIFT)),
            Some(AppCommand::CycleFocusBackward)
        );
    }

    #[test]
    fn backtab_cycles_focus_backward() {
        assert_eq!(
            command_for_key(key(KeyCode::BackTab, KeyModifiers::SHIFT)),
            Some(AppCommand::CycleFocusBackward)
        );
    }

    #[test]
    fn alt_letter_commands_are_case_insensitive() {
        assert_eq!(
            command_for_key(key(
                KeyCode::Char('T'),
                KeyModifiers::ALT | KeyModifiers::SHIFT
            )),
            Some(AppCommand::NewSurface)
        );
    }

    #[test]
    fn alt_arrows_map_to_focus_commands() {
        assert_eq!(
            command_for_key(key(KeyCode::Left, KeyModifiers::ALT)),
            Some(AppCommand::FocusLeft)
        );
        assert_eq!(
            command_for_key(key(KeyCode::Down, KeyModifiers::ALT)),
            Some(AppCommand::FocusDown)
        );
    }

    #[test]
    fn alt_surface_navigation_commands() {
        assert_eq!(
            command_for_key(key(KeyCode::Char('['), KeyModifiers::ALT)),
            Some(AppCommand::PrevSurface)
        );
        assert_eq!(
            command_for_key(key(KeyCode::Char(']'), KeyModifiers::ALT)),
            Some(AppCommand::NextSurface)
        );
    }

    #[test]
    fn alt_digit_switches_workspace() {
        assert_eq!(
            command_for_key(key(KeyCode::Char('3'), KeyModifiers::ALT)),
            Some(AppCommand::SwitchWorkspace(2))
        );
    }

    #[test]
    fn non_alt_keys_are_ignored() {
        assert_eq!(
            command_for_key(key(KeyCode::Char('d'), KeyModifiers::empty())),
            None
        );
    }

    #[test]
    fn unsupported_alt_keys_are_ignored() {
        assert_eq!(command_for_key(key(KeyCode::F(5), KeyModifiers::ALT)), None);
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }
}

```

## src/terminal/mod.rs
`ust
pub mod input;
pub mod pty;

```

## src/terminal/pty.rs
`ust
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

```

## src/ui/layout.rs
`ust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};

use crate::ui::sidebar::SIDEBAR_WIDTH;
use crate::SplitDirection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaneLayout {
    pub outer: Rect,
    pub tabbar: Option<Rect>,
    pub content: Rect,
}

pub struct LayoutState {
    pub sidebar: Option<Rect>,
    pub pane_layouts: Vec<PaneLayout>,
    pub status_area: Rect,
}

pub fn compute_layout(
    area: Rect,
    show_sidebar: bool,
    pane_surface_counts: &[usize],
    split: SplitDirection,
) -> LayoutState {
    let (sidebar, main_area) = if show_sidebar {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(1)])
            .split(area);
        (Some(columns[0]), columns[1])
    } else {
        (None, area)
    };

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(main_area);

    let pane_area = sections[0];
    let status_area = sections[1];

    let pane_layouts = if pane_surface_counts.is_empty() {
        Vec::new()
    } else {
        let direction = match split {
            SplitDirection::Vertical => Direction::Horizontal,
            SplitDirection::Horizontal => Direction::Vertical,
        };
        let pane_count = pane_surface_counts.len();
        let constraints = (0..pane_count)
            .map(|_| Constraint::Ratio(1, pane_count as u32))
            .collect::<Vec<_>>();

        Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(pane_area)
            .iter()
            .copied()
            .zip(pane_surface_counts.iter().copied())
            .map(|(outer, surface_count)| pane_layout(outer, surface_count))
            .collect()
    };

    LayoutState {
        sidebar,
        pane_layouts,
        status_area,
    }
}

fn pane_layout(outer: Rect, surface_count: usize) -> PaneLayout {
    let inner = Block::default().borders(Borders::ALL).inner(outer);
    let (tabbar, content) = if surface_count >= 2 && inner.height > 0 {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(inner);
        (Some(sections[0]), sections[1])
    } else {
        (None, inner)
    };

    PaneLayout {
        outer,
        tabbar,
        content,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::compute_layout;
    use crate::ui::sidebar::SIDEBAR_WIDTH;
    use crate::SplitDirection;

    #[test]
    fn split_two_vertical() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1, 1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.pane_layouts[0].outer.width, 60);
        assert_eq!(layout.pane_layouts[1].outer.width, 60);
        assert_eq!(layout.pane_layouts[0].outer.height, 39);
    }

    #[test]
    fn split_single_fullscreen() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.pane_layouts[0].outer.width, 120);
        assert_eq!(layout.pane_layouts[0].outer.height, 39);
    }

    #[test]
    fn layout_zero_panes() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[],
            SplitDirection::Vertical,
        );
        assert!(layout.pane_layouts.is_empty());
        assert_eq!(layout.status_area.height, 1);
        assert_eq!(layout.status_area.width, 120);
    }

    #[test]
    fn layout_two_panes_horizontal_split() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1, 1],
            SplitDirection::Horizontal,
        );

        assert_eq!(layout.pane_layouts.len(), 2);
        assert_eq!(layout.pane_layouts[0].outer.width, 120);
        assert_eq!(layout.pane_layouts[1].outer.width, 120);
    }

    #[test]
    fn layout_three_panes_even_split() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1, 1, 1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.pane_layouts[0].outer.width, 40);
        assert_eq!(layout.pane_layouts[1].outer.width, 40);
        assert_eq!(layout.pane_layouts[2].outer.width, 40);
    }

    #[test]
    fn layout_content_is_inner_without_tabbar() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1],
            SplitDirection::Vertical,
        );

        assert_eq!(
            layout.pane_layouts[0].content.width,
            layout.pane_layouts[0].outer.width.saturating_sub(2)
        );
        assert_eq!(
            layout.pane_layouts[0].content.height,
            layout.pane_layouts[0].outer.height.saturating_sub(2)
        );
        assert!(layout.pane_layouts[0].tabbar.is_none());
    }

    #[test]
    fn layout_status_bar_always_one_row() {
        for pane_count in 0..5 {
            let counts = vec![1; pane_count];
            let layout = compute_layout(
                Rect::new(0, 0, 80, 24),
                false,
                &counts,
                SplitDirection::Vertical,
            );
            assert_eq!(layout.status_area.height, 1);
        }
    }

    #[test]
    fn layout_with_sidebar_reserves_fixed_width() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            true,
            &[1, 1],
            SplitDirection::Vertical,
        );

        assert_eq!(layout.sidebar.unwrap().width, SIDEBAR_WIDTH);
        assert_eq!(layout.status_area.width, 120 - SIDEBAR_WIDTH);
    }

    #[test]
    fn layout_without_sidebar_uses_full_width() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[1],
            SplitDirection::Vertical,
        );

        assert!(layout.sidebar.is_none());
        assert_eq!(layout.status_area.width, 120);
    }

    #[test]
    fn pane_tabbar_appears_when_multiple_surfaces_exist() {
        let layout = compute_layout(
            Rect::new(0, 0, 120, 40),
            false,
            &[2],
            SplitDirection::Vertical,
        );

        let pane = layout.pane_layouts[0];
        assert_eq!(pane.tabbar.unwrap().height, 1);
        assert_eq!(pane.content.height, pane.outer.height.saturating_sub(3));
    }
}

```

## src/ui/mod.rs
`ust
pub mod layout;
pub mod sidebar;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::pane::Pane;
use crate::ui::layout::LayoutState;
use crate::workspace::Workspace;

pub fn render(frame: &mut Frame, app: &App, layout: &LayoutState) {
    if let Some(sidebar_area) = layout.sidebar {
        sidebar::render(frame, app, sidebar_area);
    }

    if app.workspaces_empty() {
        let empty = Paragraph::new("No workspaces. Press Alt+C to create one.")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(empty, main_content_area(frame.area(), layout));
        render_status_bar(frame, app, layout.status_area);
        return;
    }

    if let Some(workspace) = app.current_workspace() {
        render_workspace(frame, workspace, layout);
    }

    render_status_bar(frame, app, layout.status_area);
}

fn render_workspace(frame: &mut Frame, workspace: &Workspace, layout: &LayoutState) {
    for (index, (pane, pane_layout)) in workspace
        .panes
        .iter()
        .zip(layout.pane_layouts.iter())
        .enumerate()
    {
        let is_focused = index == workspace.focused_pane;
        let current_surface = pane.current_surface();
        let focus_color = current_surface
            .and_then(|surface| surface.agent.accent_color)
            .unwrap_or(Color::Cyan);
        let border_color = if is_focused {
            focus_color
        } else {
            Color::DarkGray
        };
        let status = if current_surface.is_some_and(|surface| surface.agent.is_dead()) {
            "dead"
        } else if current_surface.is_some_and(|surface| surface.agent.is_idle()) {
            "idle"
        } else {
            "live"
        };
        let title_label = current_surface
            .map(|surface| surface.label.as_str())
            .unwrap_or("empty");
        let title = format!(" {} [{}] ", title_label, status);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        frame.render_widget(block, pane_layout.outer);

        if let Some(tabbar_area) = pane_layout.tabbar {
            render_surface_tabbar(frame, pane, tabbar_area, focus_color);
        }

        if let Some(surface) = current_surface {
            render_vt100_screen(frame, &surface.agent.parser, pane_layout.content);
        }
    }
}

fn render_surface_tabbar(
    frame: &mut Frame,
    pane: &Pane,
    area: ratatui::layout::Rect,
    accent: Color,
) {
    let mut spans = Vec::new();

    for (index, surface) in pane.surfaces.iter().enumerate() {
        let style = if index == pane.current_surface {
            Style::default()
                .fg(Color::Black)
                .bg(accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(format!("[{}] ", surface.label), style));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn main_content_area(
    frame_area: ratatui::layout::Rect,
    layout: &LayoutState,
) -> ratatui::layout::Rect {
    let sidebar_width = layout.sidebar.map_or(0, |sidebar| sidebar.width);
    let main_x = frame_area.x.saturating_add(sidebar_width);
    let main_width = frame_area.width.saturating_sub(sidebar_width);
    let main_height = layout.status_area.y.saturating_sub(frame_area.y);

    ratatui::layout::Rect::new(main_x, frame_area.y, main_width, main_height)
}

fn render_vt100_screen(frame: &mut Frame, parser: &vt100::Parser, area: ratatui::layout::Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let screen = parser.screen();
    let mut lines = Vec::with_capacity(area.height as usize);

    for row in 0..area.height {
        let mut spans = Vec::with_capacity(area.width as usize);

        for col in 0..area.width {
            if let Some(cell) = screen.cell(row, col) {
                let contents = cell.contents();
                let text = if contents.is_empty() {
                    " ".to_string()
                } else {
                    contents
                };

                let mut fg = vt100_color_to_ratatui(cell.fgcolor());
                let mut bg = vt100_color_to_ratatui(cell.bgcolor());

                if cell.inverse() {
                    std::mem::swap(&mut fg, &mut bg);
                }

                let mut style = Style::default().fg(fg).bg(bg);

                if cell.bold() {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cell.italic() {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cell.underline() {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }

                spans.push(Span::styled(text, style));
            } else {
                spans.push(Span::raw(" "));
            }
        }

        lines.push(Line::from(spans));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn vt100_color_to_ratatui(color: vt100::Color) -> Color {
    match color {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(index) => Color::Indexed(index),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut spans = vec![
        Span::styled(
            " agx ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];

    if let Some(workspace) = app.current_workspace() {
        spans.push(Span::styled(
            format!(" ws:{} ", workspace.name),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ));
        spans.push(Span::raw(" "));

        spans.push(Span::styled(
            format!(" panes:{} ", workspace.panes.len()),
            Style::default().fg(Color::Gray),
        ));
        spans.push(Span::raw(" "));

        if let Some(pane) = workspace.focused_pane() {
            let current_surface = if pane.surfaces.is_empty() {
                0
            } else {
                pane.current_surface + 1
            };
            spans.push(Span::styled(
                format!(" surf:{}/{} ", current_surface, pane.surfaces.len()),
                Style::default().fg(Color::Gray),
            ));
            spans.push(Span::raw(" "));
        }
    }

    spans.push(Span::styled(
        " Alt+?:help Alt+D/S split Alt+T/W surface Alt+B sidebar Alt+Q quit ",
        Style::default().fg(Color::Gray),
    ));

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use ratatui::style::Color;

    use super::vt100_color_to_ratatui;

    #[test]
    fn color_default() {
        assert_eq!(vt100_color_to_ratatui(vt100::Color::Default), Color::Reset);
    }

    #[test]
    fn color_indexed() {
        assert_eq!(
            vt100_color_to_ratatui(vt100::Color::Idx(196)),
            Color::Indexed(196)
        );
    }

    #[test]
    fn color_rgb() {
        assert_eq!(
            vt100_color_to_ratatui(vt100::Color::Rgb(10, 20, 30)),
            Color::Rgb(10, 20, 30)
        );
    }
}

```

## src/ui/sidebar.rs
`ust
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub const SIDEBAR_WIDTH: u16 = 18;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let focused = app.sidebar_focused;
    let cursor = app.sidebar_cursor;

    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let mut lines = Vec::new();

    for (index, workspace) in app.workspaces.iter().enumerate() {
        let is_current = index == app.current_workspace;
        let is_cursor = focused && index == cursor;

        let (marker, name_style) = if is_cursor {
            (
                ">",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_current {
            (
                "*",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (" ", Style::default().fg(Color::White))
        };

        lines.push(Line::from(vec![Span::styled(
            format!("{marker} {}", workspace.name),
            name_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!("  panes:{}", workspace.panes.len()),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " no workspaces",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let title = if focused {
        " ^v:Move Enter:Select N:New D:Del "
    } else {
        " ws "
    };

    let sidebar = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(border_color)),
    );
    frame.render_widget(sidebar, area);
}

#[cfg(test)]
mod tests {
    use super::SIDEBAR_WIDTH;

    #[test]
    fn sidebar_width_matches_spec() {
        assert_eq!(SIDEBAR_WIDTH, 18);
    }
}

```

## src/workspace.rs
`ust
use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::agent::PaneSpec;
use crate::pane::Pane;
use crate::SplitDirection;

pub struct Workspace {
    pub name: String,
    pub panes: Vec<Pane>,
    pub focused_pane: usize,
    pub split: SplitDirection,
}

impl Workspace {
    pub fn new(name: impl Into<String>, spec: PaneSpec, split: SplitDirection) -> Result<Self> {
        Ok(Self {
            name: name.into(),
            panes: vec![Pane::new(spec)?],
            focused_pane: 0,
            split,
        })
    }

    pub fn poll(&mut self) {
        for pane in &mut self.panes {
            pane.poll();
        }
    }

    pub fn split_right(&mut self, spec: PaneSpec) -> Result<()> {
        self.split_with(spec, SplitDirection::Vertical)
    }

    pub fn split_down(&mut self, spec: PaneSpec) -> Result<()> {
        self.split_with(spec, SplitDirection::Horizontal)
    }

    pub fn close_focused_pane(&mut self) {
        if self.panes.is_empty() {
            return;
        }

        self.panes.remove(self.focused_pane);
        if self.focused_pane >= self.panes.len() && !self.panes.is_empty() {
            self.focused_pane = self.panes.len() - 1;
        } else if self.panes.is_empty() {
            self.focused_pane = 0;
        }
    }

    pub fn focus_left(&mut self) {
        if self.split == SplitDirection::Vertical && self.focused_pane > 0 {
            self.focused_pane -= 1;
        }
    }

    pub fn focus_right(&mut self) {
        if self.split == SplitDirection::Vertical && self.focused_pane + 1 < self.panes.len() {
            self.focused_pane += 1;
        }
    }

    pub fn focus_up(&mut self) {
        if self.split == SplitDirection::Horizontal && self.focused_pane > 0 {
            self.focused_pane -= 1;
        }
    }

    pub fn focus_down(&mut self) {
        if self.split == SplitDirection::Horizontal && self.focused_pane + 1 < self.panes.len() {
            self.focused_pane += 1;
        }
    }

    pub fn add_surface_to_focused(&mut self, spec: PaneSpec) -> Result<()> {
        if let Some(pane) = self.focused_pane_mut() {
            pane.add_surface(spec)?;
        } else {
            self.panes.push(Pane::new(spec)?);
            self.focused_pane = 0;
        }
        Ok(())
    }

    pub fn close_current_surface(&mut self) {
        let Some(index) = self.focused_pane_index() else {
            return;
        };

        if let Some(pane) = self.panes.get_mut(index) {
            pane.close_current_surface();
        }

        if self.panes.get(index).is_some_and(Pane::is_empty) {
            self.close_focused_pane();
        }
    }

    pub fn next_surface(&mut self) {
        if let Some(pane) = self.focused_pane_mut() {
            pane.next_surface();
        }
    }

    pub fn prev_surface(&mut self) {
        if let Some(pane) = self.focused_pane_mut() {
            pane.prev_surface();
        }
    }

    pub fn focused_pane(&self) -> Option<&Pane> {
        self.panes.get(self.focused_pane)
    }

    pub fn focused_pane_mut(&mut self) -> Option<&mut Pane> {
        self.panes.get_mut(self.focused_pane)
    }

    pub fn send_key_to_focused(&mut self, key: KeyEvent) {
        if let Some(pane) = self.focused_pane_mut() {
            pane.send_key(key);
        }
    }

    pub fn pane_surface_counts(&self) -> Vec<usize> {
        self.panes.iter().map(|pane| pane.surfaces.len()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }

    fn split_with(&mut self, spec: PaneSpec, split: SplitDirection) -> Result<()> {
        self.split = split;
        self.panes.push(Pane::new(spec)?);
        self.focused_pane = self.panes.len().saturating_sub(1);
        Ok(())
    }

    fn focused_pane_index(&self) -> Option<usize> {
        if self.panes.is_empty() {
            None
        } else {
            Some(self.focused_pane.min(self.panes.len() - 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Workspace;
    use crate::agent::PaneSpec;
    use crate::SplitDirection;

    fn workspace(split: SplitDirection) -> Workspace {
        Workspace::new("my-ws", interactive_shell_spec("surface-1"), split).unwrap()
    }

    #[test]
    fn split_right_adds_pane_and_focuses_it() {
        let mut ws = workspace(SplitDirection::Vertical);
        ws.split_right(interactive_shell_spec("surface-2")).unwrap();
        assert_eq!(ws.panes.len(), 2);
        assert_eq!(ws.focused_pane, 1);
        assert_eq!(ws.split, SplitDirection::Vertical);
    }

    #[test]
    fn split_down_updates_split() {
        let mut ws = workspace(SplitDirection::Vertical);
        ws.split_down(interactive_shell_spec("surface-2")).unwrap();
        assert_eq!(ws.panes.len(), 2);
        assert_eq!(ws.focused_pane, 1);
        assert_eq!(ws.split, SplitDirection::Horizontal);
    }

    #[test]
    fn focus_left_and_right_follow_vertical_split() {
        let mut ws = workspace(SplitDirection::Vertical);
        ws.split_right(interactive_shell_spec("surface-2")).unwrap();
        ws.focus_left();
        assert_eq!(ws.focused_pane, 0);
        ws.focus_right();
        assert_eq!(ws.focused_pane, 1);
    }

    #[test]
    fn focus_up_and_down_follow_horizontal_split() {
        let mut ws = workspace(SplitDirection::Horizontal);
        ws.split_down(interactive_shell_spec("surface-2")).unwrap();
        ws.focus_up();
        assert_eq!(ws.focused_pane, 0);
        ws.focus_down();
        assert_eq!(ws.focused_pane, 1);
    }

    #[test]
    fn close_focused_pane_is_noop_on_empty() {
        let mut ws = Workspace {
            name: "empty".to_string(),
            panes: Vec::new(),
            focused_pane: 0,
            split: SplitDirection::Vertical,
        };
        ws.close_focused_pane();
        assert!(ws.is_empty());
        assert_eq!(ws.focused_pane, 0);
    }

    #[test]
    fn close_current_surface_removes_empty_pane() {
        let mut ws = workspace(SplitDirection::Vertical);
        ws.close_current_surface();
        assert!(ws.is_empty());
    }

    #[test]
    fn add_surface_targets_focused_pane() {
        let mut ws = workspace(SplitDirection::Vertical);
        ws.add_surface_to_focused(interactive_shell_spec("surface-2"))
            .unwrap();
        assert_eq!(ws.focused_pane().unwrap().surfaces.len(), 2);
        assert_eq!(ws.focused_pane().unwrap().current_surface, 1);
    }

    #[test]
    fn workspace_is_empty_when_no_panes() {
        let ws = Workspace {
            name: "test".to_string(),
            panes: Vec::new(),
            focused_pane: 0,
            split: SplitDirection::Vertical,
        };
        assert!(ws.is_empty());
    }

    #[test]
    fn workspace_name() {
        let ws = workspace(SplitDirection::Horizontal);
        assert_eq!(ws.name, "my-ws");
        assert_eq!(ws.split, SplitDirection::Horizontal);
    }

    fn interactive_shell_spec(label: &str) -> PaneSpec {
        let command = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };
        PaneSpec::new(label.to_string(), command.to_string())
    }
}

```
