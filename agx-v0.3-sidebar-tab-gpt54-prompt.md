# agx v0.3 사이드바 + Tab 모델

## 1. 계층 구조 변경

```text
현재:  App  Workspace  Pane
변경:  App  Workspace  Tab  Pane
```

## 2. 화면 레이아웃

```text
 [tab1] [tab2] [tab3] 
                                             
  ws1     
 >ws2                                       
  ws3        pane1          pane2           
             [live]         [idle]          
                                            
                                            
                                            

 agx  1:ws2  tab:2/3  panes:2  Ctrl-a:help   
```

## 3. 새 파일 구조

```text
src/
 tab.rs              # 새로 추가
 workspace.rs        # Tab 포함하도록 수정
 ui/
    mod.rs          # 전체 레이아웃 조합
    layout.rs       # 레이아웃 계산 수정
    sidebar.rs      # 새로 추가
    tabbar.rs       # 새로 추가
```

## 4. 모델 설계

**src/tab.rs**
```rust
pub struct Tab {
    pub name: String,
    pub panes: Vec<AgentPane>,
    pub focused: usize,
    pub split: SplitDirection,
}
```

**src/workspace.rs 수정**
```rust
pub struct Workspace {
    pub name: String,
    pub tabs: Vec<Tab>,        // Vec<AgentPane> -> Vec<Tab>
    pub current_tab: usize,    // 추가
}
```

## 5. 키바인딩 추가

| 키 | 동작 |
|----|------|
| prefix + `T` | 현재 워크스페이스에 새 탭 생성 |
| prefix + `W` | 현재 탭 닫기 (확인 프롬프트) |
| prefix + `[` | 이전 탭으로 전환 |
| prefix + `]` | 다음 탭으로 전환 |
| prefix + `B` | 사이드바 토글 |
| prefix + `N` | 현재 탭에 새 패인 추가 (기존 유지) |
| prefix + `X` | 포커스된 패인 닫기 (기존 유지) |

## 6. 레이아웃 계산

```text
전체 영역 (120x40)
 사이드바: 고정 18 컬럼 (토글 가능)
 메인 영역: 나머지
     탭바: 1줄
     패인 영역: 나머지
     상태바: 1줄
```

```rust
// ui/layout.rs
pub struct LayoutState {
    pub sidebar: Option<Rect>,   // None이면 숨김
    pub tabbar: Rect,            // 새로 추가
    pub pane_areas: Vec<Rect>,
    pub pane_inners: Vec<Rect>,
    pub status_area: Rect,
}
```

## 7. GPT-5.4 작업 지시

```markdown
# agx v0.3 사이드바 + Tab 모델

## 프로젝트 컨텍스트
- agx = 크로스플랫폼 AI 에이전트 터미널 멀티플렉서
- Rust + ratatui 0.29 + crossterm 0.28 + tokio + portable-pty + vt100
- crates.io 등록 완료. MIT OR Apache-2.0
- 현재 상태: cargo test 68 passed, clippy clean

## 현재 소스 구조
- main.rs          # CLI 엔트리포인트 (clap)
- app.rs           # 이벤트 루프, prefix key, 워크스페이스 관리
- config.rs / config/loader.rs  # config.toml 파싱
- workspace.rs     # Workspace (현재 panes 직접 보유)
- agent/mod.rs     # AgentPane: PTY spawn, reader 스레드+채널, vt100, encode_key
- terminal/pty.rs  # PtyProcess: portable-pty 래퍼
- terminal/mod.rs / terminal/input.rs
- ui/mod.rs        # compute_layout, render, vt100->ratatui, 상태바
- ui/layout.rs     # 레이아웃 계산

## v0.3 요구사항

### 1. Tab 모델 추가
- 새 파일: src/tab.rs
- Tab 구조체: name, panes: Vec<AgentPane>, focused: usize, split: SplitDirection
- Workspace가 Vec<Tab>을 가지도록 수정 (기존 Vec<AgentPane> 대체)
- Workspace에 current_tab: usize 추가
- 기존 패인 관련 메서드(add_pane, close_focused_pane, focus_prev/next, send_key_to_focused, poll)를 Tab으로 이동
- Workspace는 Tab을 관리하는 메서드 추가: add_tab, close_current_tab, next_tab, prev_tab

### 2. 사이드바 위젯
- 새 파일: src/ui/sidebar.rs
- 왼쪽 고정 18 컬럼
- 워크스페이스 목록 표시 (현재 선택은 하이라이트)
- 각 워크스페이스 아래에 탭 수 표시
- prefix + B로 토글 (app.rs에 show_sidebar: bool 추가)

### 3. 탭바 위젯
- 새 파일: src/ui/tabbar.rs
- 사이드바 옆 상단 1줄
- 현재 워크스페이스의 탭 목록 표시
- 현재 탭은 하이라이트

### 4. 레이아웃 수정
- ui/layout.rs 또는 ui/mod.rs의 compute_layout 수정
- LayoutState에 sidebar: Option<Rect>, tabbar: Rect 추가
- 사이드바 표시 시: 전체에서 왼쪽 18컬럼 분리
- 나머지에서 상단 1줄 = 탭바, 하단 1줄 = 상태바, 중간 = 패인 영역

### 5. 키바인딩 추가 (app.rs handle_prefix_key 수정)
- T: 현재 워크스페이스에 새 탭 생성
- W: 현재 탭 닫기
- [: 이전 탭
- ]: 다음 탭
- B: 사이드바 토글

### 6. 테스트 추가
- tab.rs: focus_prev/next, close_focused_pane, is_empty (workspace.rs 기존 테스트와 동일 패턴)
- ui/sidebar.rs: 사이드바 렌더/영역 관련 테스트
- ui/tabbar.rs: 탭바 렌더/영역 관련 테스트
- workspace.rs: add_tab, close_current_tab, next_tab, prev_tab
- layout: 사이드바 있을 때/없을 때 영역 계산

## 제약사항
- 기존 테스트 깨지면 안 됨
- cargo test + cargo clippy 통과 필수
- 기존 키바인딩 (N, X, C, 1~9, Q, 방향키) 유지
- 크로스플랫폼 유지

## 출력 형식
- 변경/추가되는 파일별로 전체 코드 제공
- 파일 경로 명확히 표기
```

## 현재 소스 파일 전체
아래는 이 워크스페이스의 현재 `src/` 전체 파일 전문입니다. 특히 `app.rs`, `workspace.rs`, `ui/mod.rs`는 반드시 그대로 참고해서 기존 코드와 충돌 없이 수정하세요.

## src\agent\detector.rs
```rust
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


## src\agent\mod.rs
```rust
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


## src\agent\process.rs
```rust
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


## src\agent\registry.rs
```rust
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


## src\app.rs
```rust
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::config::loader::Config;
use crate::terminal::input::{InputAction, PrefixCommand, PrefixRouter};
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
    pub confirm_close_pane: bool,
    config: Config,
    input_router: PrefixRouter,
    next_workspace_id: usize,
    default_split: SplitDirection,
}

impl App {
    pub fn new(options: AppOptions) -> Result<Self> {
        let config = Config::load()?;
        let prefix_binding = config.prefix_binding()?;
        let default_split = options.split.unwrap_or(config.default_split()?);

        let mut initial_panes = Vec::new();
        if options.run.is_empty() {
            initial_panes.push(crate::agent::AgentPane::spawn(config.default_pane_spec()?)?);
        } else {
            for command in options.run {
                initial_panes.push(crate::agent::AgentPane::spawn(
                    config.resolve_pane_spec(&command)?,
                )?);
            }
        }

        Ok(Self {
            workspaces: vec![Workspace::with_panes("ws1", default_split, initial_panes)],
            current_workspace: 0,
            should_quit: false,
            confirm_close_pane: false,
            config,
            input_router: PrefixRouter::new(prefix_binding, Duration::from_secs(2)),
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

            let _ = self.input_router.expire(Instant::now());

            for workspace in &mut self.workspaces {
                workspace.poll();
            }

            terminal.draw(|frame| {
                let (pane_count, split) = match self.current_workspace() {
                    Some(workspace) => (workspace.panes.len(), workspace.split),
                    None => (0, self.default_split),
                };

                let layout = ui::layout::compute_layout(frame.area(), pane_count, split);
                self.resize_current_workspace(&layout);
                ui::render(frame, self, &layout);
            })?;

            if event::poll(Duration::from_millis(50))? {
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

        for (pane, area) in workspace.panes.iter_mut().zip(&layout.pane_inners) {
            pane.resize(area.height.max(1), area.width.max(1));
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.confirm_close_pane {
            return self.handle_close_confirmation(key);
        }

        match self.input_router.route_key(key, Instant::now()) {
            InputAction::ForwardToPty(key) => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.send_key_to_focused(key);
                }
            }
            InputAction::EnterPrefixMode | InputAction::PrefixTimedOut | InputAction::Noop => {}
            InputAction::PrefixCommand(command) => {
                self.handle_prefix_command(command)?;
            }
        }

        Ok(())
    }

    fn handle_prefix_command(&mut self, command: PrefixCommand) -> Result<()> {
        match command {
            PrefixCommand::Quit => self.should_quit = true,
            PrefixCommand::NewPane => self.add_pane_to_current_workspace()?,
            PrefixCommand::ClosePane => {
                if self
                    .current_workspace()
                    .is_some_and(|workspace| !workspace.panes.is_empty())
                {
                    self.confirm_close_pane = true;
                }
            }
            PrefixCommand::NewWorkspace => self.create_workspace()?,
            PrefixCommand::SwitchWorkspace(index) => {
                if index < self.workspaces.len() {
                    self.current_workspace = index;
                }
            }
            PrefixCommand::FocusPrev => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.focus_prev();
                }
            }
            PrefixCommand::FocusNext => {
                if let Some(workspace) = self.current_workspace_mut() {
                    workspace.focus_next();
                }
            }
        }

        Ok(())
    }

    fn handle_close_confirmation(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'y') => {
                self.confirm_close_pane = false;
                self.close_focused_pane();
            }
            KeyCode::Char(c) if c.eq_ignore_ascii_case(&'n') => {
                self.confirm_close_pane = false;
            }
            KeyCode::Esc => {
                self.confirm_close_pane = false;
            }
            _ => {}
        }

        Ok(())
    }

    fn add_pane_to_current_workspace(&mut self) -> Result<()> {
        let spec = self.config.default_pane_spec()?;
        if let Some(workspace) = self.current_workspace_mut() {
            workspace.add_pane(spec)?;
        } else {
            self.create_workspace()?;
        }
        Ok(())
    }

    fn create_workspace(&mut self) -> Result<()> {
        let name = format!("ws{}", self.next_workspace_id);
        self.next_workspace_id += 1;

        let spec = self.config.default_pane_spec()?;
        let mut workspace = Workspace::with_panes(name, self.default_split, Vec::new());
        workspace.add_pane(spec)?;
        self.workspaces.push(workspace);
        self.current_workspace = self.workspaces.len().saturating_sub(1);
        Ok(())
    }

    fn close_focused_pane(&mut self) {
        let Some(workspace) = self.current_workspace_mut() else {
            return;
        };

        workspace.close_focused_pane();
        if workspace.is_empty() {
            let index = self.current_workspace;
            self.workspaces.remove(index);
            if self.current_workspace >= self.workspaces.len() && !self.workspaces.is_empty() {
                self.current_workspace = self.workspaces.len() - 1;
            } else if self.workspaces.is_empty() {
                self.current_workspace = 0;
            }
        }
    }

    pub fn current_workspace(&self) -> Option<&Workspace> {
        self.workspaces.get(self.current_workspace)
    }

    fn current_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspaces.get_mut(self.current_workspace)
    }

    pub fn prefix_is_active(&self) -> bool {
        self.input_router.is_prefix_active()
    }

    pub fn prefix_label(&self) -> &str {
        self.input_router.binding_label()
    }

    pub fn prefix_timeout_seconds(&self) -> u64 {
        self.input_router.timeout().as_secs()
    }

    pub fn workspaces_empty(&self) -> bool {
        self.workspaces.is_empty()
    }
}

```


## src\config.rs
```rust
pub mod loader;

pub use loader::{AgentConfig, Config, DefaultsConfig, KeybindConfig};

```


## src\config\loader.rs
```rust
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use ratatui::style::Color;
use serde::Deserialize;

use crate::agent::registry::AgentDefinition;
use crate::agent::PaneSpec;
use crate::terminal::input::KeyBinding;
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

#[derive(Clone, Debug, Deserialize)]
pub struct KeybindConfig {
    #[serde(default = "default_prefix")]
    pub prefix: String,
}

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

    pub fn prefix_binding(&self) -> Result<KeyBinding> {
        KeyBinding::parse(&self.keybind.prefix)
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

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            prefix: default_prefix(),
        }
    }
}

fn default_prefix() -> String {
    "Ctrl-a".to_string()
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
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::style::Color;

    use super::Config;
    use crate::SplitDirection;

    #[test]
    fn parse_valid_config() {
        let config = Config::load_from_str(
            r#"
[keybind]
prefix = "Ctrl-a"

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
        assert!(config
            .prefix_binding()
            .unwrap()
            .matches(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)));
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
}

```


## src\lib.rs
```rust
pub mod agent;
pub mod app;
pub mod config;
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


## src\main.rs
```rust
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


## src\terminal\input.rs
```rust
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyBinding {
    raw: String,
    modifiers: KeyModifiers,
    code: KeyCode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefixCommand {
    FocusPrev,
    FocusNext,
    NewPane,
    ClosePane,
    NewWorkspace,
    SwitchWorkspace(usize),
    Quit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InputAction {
    ForwardToPty(KeyEvent),
    EnterPrefixMode,
    PrefixCommand(PrefixCommand),
    PrefixTimedOut,
    Noop,
}

pub struct PrefixRouter {
    binding: KeyBinding,
    timeout: Duration,
    prefix_started_at: Option<Instant>,
}

impl KeyBinding {
    pub fn parse(value: &str) -> Result<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            bail!("prefix key cannot be empty");
        }

        let tokens = trimmed
            .split('-')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            bail!("invalid prefix key `{trimmed}`");
        }

        let mut modifiers = KeyModifiers::empty();
        for token in &tokens[..tokens.len().saturating_sub(1)] {
            match token.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "alt" => modifiers |= KeyModifiers::ALT,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "super" | "cmd" | "command" | "meta" => modifiers |= KeyModifiers::SUPER,
                other => bail!("unsupported modifier `{other}` in prefix key `{trimmed}`"),
            }
        }

        let code = parse_key_code(tokens[tokens.len() - 1])?;
        Ok(Self {
            raw: trimmed.to_string(),
            modifiers,
            code,
        })
    }

    pub fn matches(&self, key: KeyEvent) -> bool {
        if key.modifiers != self.modifiers {
            return false;
        }

        match (self.code, key.code) {
            (KeyCode::Char(expected), KeyCode::Char(actual)) => {
                expected.eq_ignore_ascii_case(&actual)
            }
            _ => self.code == key.code,
        }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

impl PrefixRouter {
    pub fn new(binding: KeyBinding, timeout: Duration) -> Self {
        Self {
            binding,
            timeout,
            prefix_started_at: None,
        }
    }

    pub fn route_key(&mut self, key: KeyEvent, now: Instant) -> InputAction {
        let _ = self.expire(now);

        if self.binding.matches(key) {
            self.prefix_started_at = Some(now);
            return InputAction::EnterPrefixMode;
        }

        if self.is_prefix_active() {
            self.prefix_started_at = None;
            return map_prefix_key(key);
        }

        InputAction::ForwardToPty(key)
    }

    pub fn expire(&mut self, now: Instant) -> Option<InputAction> {
        if self
            .prefix_started_at
            .is_some_and(|started| now.duration_since(started) >= self.timeout)
        {
            self.prefix_started_at = None;
            return Some(InputAction::PrefixTimedOut);
        }

        None
    }

    pub fn is_prefix_active(&self) -> bool {
        self.prefix_started_at.is_some()
    }

    pub fn binding_label(&self) -> &str {
        self.binding.raw()
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

fn parse_key_code(token: &str) -> Result<KeyCode> {
    let lower = token.to_ascii_lowercase();
    match lower.as_str() {
        "enter" => Ok(KeyCode::Enter),
        "esc" | "escape" => Ok(KeyCode::Esc),
        "tab" => Ok(KeyCode::Tab),
        "space" => Ok(KeyCode::Char(' ')),
        other if other.chars().count() == 1 => Ok(KeyCode::Char(other.chars().next().unwrap())),
        _ => bail!("unsupported key `{token}` in prefix binding"),
    }
}

fn map_prefix_key(key: KeyEvent) -> InputAction {
    match key.code {
        KeyCode::Left | KeyCode::Up => InputAction::PrefixCommand(PrefixCommand::FocusPrev),
        KeyCode::Right | KeyCode::Down => InputAction::PrefixCommand(PrefixCommand::FocusNext),
        KeyCode::Char(c) => match c.to_ascii_lowercase() {
            'n' => InputAction::PrefixCommand(PrefixCommand::NewPane),
            'x' => InputAction::PrefixCommand(PrefixCommand::ClosePane),
            'c' => InputAction::PrefixCommand(PrefixCommand::NewWorkspace),
            'q' => InputAction::PrefixCommand(PrefixCommand::Quit),
            digit if ('1'..='9').contains(&digit) => InputAction::PrefixCommand(
                PrefixCommand::SwitchWorkspace(digit as usize - '1' as usize),
            ),
            _ => InputAction::Noop,
        },
        _ => InputAction::Noop,
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{InputAction, KeyBinding, PrefixRouter};

    #[test]
    fn parse_prefix_ctrl_a() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert_eq!(binding.modifiers, KeyModifiers::CONTROL);
        assert_eq!(binding.code, KeyCode::Char('a'));
    }

    #[test]
    fn parse_prefix_alt_shift() {
        let binding = KeyBinding::parse("Alt-Shift-x").unwrap();
        assert!(binding.modifiers.contains(KeyModifiers::ALT));
        assert!(binding.modifiers.contains(KeyModifiers::SHIFT));
        assert_eq!(binding.code, KeyCode::Char('x'));
    }

    #[test]
    fn parse_prefix_empty_fails() {
        assert!(KeyBinding::parse("").is_err());
    }

    #[test]
    fn parse_prefix_invalid_modifier() {
        assert!(KeyBinding::parse("Hyper-a").is_err());
    }

    #[test]
    fn keybinding_matches_correct_key() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert!(binding.matches(key(KeyCode::Char('a'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn keybinding_rejects_wrong_modifier() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert!(!binding.matches(key(KeyCode::Char('a'), KeyModifiers::ALT)));
    }

    #[test]
    fn keybinding_case_insensitive() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        assert!(binding.matches(key(KeyCode::Char('A'), KeyModifiers::CONTROL)));
    }

    #[test]
    fn normal_mode_forwards_key() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        let mut router = PrefixRouter::new(binding, Duration::from_secs(2));
        let action = router.route_key(
            key(KeyCode::Char('z'), KeyModifiers::empty()),
            Instant::now(),
        );

        assert!(matches!(action, InputAction::ForwardToPty(_)));
    }

    #[test]
    fn prefix_mode_enter() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        let mut router = PrefixRouter::new(binding, Duration::from_secs(2));
        let action = router.route_key(
            key(KeyCode::Char('a'), KeyModifiers::CONTROL),
            Instant::now(),
        );

        assert_eq!(action, InputAction::EnterPrefixMode);
        assert!(router.is_prefix_active());
    }

    #[test]
    fn prefix_mode_timeout() {
        let binding = KeyBinding::parse("Ctrl-a").unwrap();
        let mut router = PrefixRouter::new(binding, Duration::from_secs(2));
        let now = Instant::now();
        let _ = router.route_key(key(KeyCode::Char('a'), KeyModifiers::CONTROL), now);
        let timed_out = router.expire(now + Duration::from_secs(2));

        assert_eq!(timed_out, Some(InputAction::PrefixTimedOut));
        assert!(!router.is_prefix_active());
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }
}

```


## src\terminal\mod.rs
```rust
pub mod input;
pub mod pty;

```


## src\terminal\pty.rs
```rust
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


## src\ui\layout.rs
```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};

use crate::SplitDirection;

pub struct LayoutState {
    pub pane_areas: Vec<Rect>,
    pub pane_inners: Vec<Rect>,
    pub status_area: Rect,
}

pub fn compute_layout(area: Rect, pane_count: usize, split: SplitDirection) -> LayoutState {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    let pane_area = sections[0];
    let status_area = sections[1];

    let pane_areas = if pane_count == 0 {
        Vec::new()
    } else {
        let direction = match split {
            SplitDirection::Vertical => Direction::Horizontal,
            SplitDirection::Horizontal => Direction::Vertical,
        };

        let constraints = (0..pane_count)
            .map(|_| Constraint::Ratio(1, pane_count as u32))
            .collect::<Vec<_>>();
        let chunks = Layout::default()
            .direction(direction)
            .constraints(constraints)
            .split(pane_area);

        chunks.iter().copied().collect::<Vec<_>>()
    };

    let pane_inners = pane_areas
        .iter()
        .map(|area| Block::default().borders(Borders::ALL).inner(*area))
        .collect();

    LayoutState {
        pane_areas,
        pane_inners,
        status_area,
    }
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;

    use super::compute_layout;
    use crate::SplitDirection;

    #[test]
    fn split_two_vertical() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 2, SplitDirection::Vertical);

        assert_eq!(layout.pane_areas[0].width, 60);
        assert_eq!(layout.pane_areas[1].width, 60);
        assert_eq!(layout.pane_areas[0].height, 39);
    }

    #[test]
    fn split_single_fullscreen() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 1, SplitDirection::Vertical);

        assert_eq!(layout.pane_areas[0].width, 120);
        assert_eq!(layout.pane_areas[0].height, 39);
    }

    #[test]
    fn layout_zero_panes() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 0, SplitDirection::Vertical);
        assert!(layout.pane_areas.is_empty());
        assert!(layout.pane_inners.is_empty());
        assert_eq!(layout.status_area.height, 1);
        assert_eq!(layout.status_area.width, 120);
    }

    #[test]
    fn layout_two_panes_horizontal_split() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 2, SplitDirection::Horizontal);
        assert_eq!(layout.pane_areas.len(), 2);
        assert_eq!(layout.pane_areas[0].width, 120);
        assert_eq!(layout.pane_areas[1].width, 120);
    }

    #[test]
    fn layout_three_panes_even_split() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 3, SplitDirection::Vertical);
        assert_eq!(layout.pane_areas[0].width, 40);
        assert_eq!(layout.pane_areas[1].width, 40);
        assert_eq!(layout.pane_areas[2].width, 40);
    }

    #[test]
    fn layout_inners_smaller_than_areas() {
        let layout = compute_layout(Rect::new(0, 0, 120, 40), 1, SplitDirection::Vertical);
        assert_eq!(
            layout.pane_inners[0].width,
            layout.pane_areas[0].width.saturating_sub(2)
        );
        assert_eq!(
            layout.pane_inners[0].height,
            layout.pane_areas[0].height.saturating_sub(2)
        );
    }

    #[test]
    fn layout_status_bar_always_one_row() {
        for pane_count in 0..5 {
            let layout = compute_layout(
                Rect::new(0, 0, 80, 24),
                pane_count,
                SplitDirection::Vertical,
            );
            assert_eq!(layout.status_area.height, 1);
        }
    }
}

```


## src\ui\mod.rs
```rust
pub mod layout;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::ui::layout::LayoutState;
use crate::workspace::Workspace;

pub fn render(frame: &mut Frame, app: &App, layout: &LayoutState) {
    let area = frame.area();

    if app.workspaces_empty() {
        let empty = Paragraph::new("No workspaces. Press prefix then C to create one.")
            .style(Style::default().fg(Color::Red));
        frame.render_widget(empty, area);
        render_status_bar(frame, app, layout.status_area);
        return;
    }

    if let Some(workspace) = app.current_workspace() {
        render_workspace(frame, workspace, layout);
    }

    render_status_bar(frame, app, layout.status_area);
}

fn render_workspace(frame: &mut Frame, workspace: &Workspace, layout: &LayoutState) {
    for (index, pane) in workspace.panes.iter().enumerate() {
        let is_focused = index == workspace.focused;
        let focus_color = pane.accent_color.unwrap_or(Color::Cyan);
        let border_color = if is_focused {
            focus_color
        } else {
            Color::DarkGray
        };
        let status = if pane.is_dead() {
            "dead"
        } else if pane.is_idle() {
            "idle"
        } else {
            "live"
        };
        let title = format!(" {} [{}] ", pane.label, status);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        frame.render_widget(block, layout.pane_areas[index]);
        render_vt100_screen(frame, &pane.parser, layout.pane_inners[index]);
    }
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

    if app.prefix_is_active() {
        spans.push(Span::styled(
            format!(" PREFIX:{} ", app.prefix_label()),
            Style::default().fg(Color::Black).bg(Color::Green),
        ));
        spans.push(Span::raw(" "));
    }

    if app.confirm_close_pane {
        spans.push(Span::styled(
            " Close pane? [y/n] ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        ));
        spans.push(Span::raw(" "));
    }

    for (index, workspace) in app.workspaces.iter().enumerate() {
        let style = if index == app.current_workspace {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };

        spans.push(Span::styled(
            format!(" {}:{} ", index + 1, workspace.name),
            style,
        ));
        spans.push(Span::raw(" "));
    }

    if let Some(workspace) = app.current_workspace() {
        spans.push(Span::styled(
            format!(" panes:{} ", workspace.panes.len()),
            Style::default().fg(Color::Gray),
        ));
        spans.push(Span::raw(" "));
    }

    spans.push(Span::styled(
        format!(
            " Prefix {} then Arrows/N/X/C/1-9/Q  timeout:{}s ",
            app.prefix_label(),
            app.prefix_timeout_seconds()
        ),
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


## src\workspace.rs
```rust
use crate::agent::{AgentPane, PaneSpec};
use crate::SplitDirection;

pub struct Workspace {
    pub name: String,
    pub panes: Vec<AgentPane>,
    pub focused: usize,
    pub split: SplitDirection,
}

impl Workspace {
    pub fn with_panes(
        name: impl Into<String>,
        split: SplitDirection,
        panes: Vec<AgentPane>,
    ) -> Self {
        Self {
            name: name.into(),
            panes,
            focused: 0,
            split,
        }
    }

    pub fn poll(&mut self) {
        for pane in &mut self.panes {
            pane.poll();
        }
    }

    pub fn add_pane(&mut self, spec: PaneSpec) -> anyhow::Result<()> {
        self.panes.push(AgentPane::spawn(spec)?);
        self.focused = self.panes.len().saturating_sub(1);
        Ok(())
    }

    pub fn close_focused_pane(&mut self) {
        if self.panes.is_empty() {
            return;
        }

        self.panes.remove(self.focused);
        if self.focused >= self.panes.len() && !self.panes.is_empty() {
            self.focused = self.panes.len() - 1;
        } else if self.panes.is_empty() {
            self.focused = 0;
        }
    }

    pub fn focus_prev(&mut self) {
        if self.focused > 0 {
            self.focused -= 1;
        }
    }

    pub fn focus_next(&mut self) {
        if self.focused + 1 < self.panes.len() {
            self.focused += 1;
        }
    }

    pub fn send_key_to_focused(&mut self, key: crossterm::event::KeyEvent) {
        if let Some(pane) = self.panes.get_mut(self.focused) {
            pane.send_key(key);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Workspace;
    use crate::SplitDirection;

    fn dummy_workspace() -> Workspace {
        Workspace::with_panes("test", SplitDirection::Vertical, Vec::new())
    }

    #[test]
    fn focus_prev_at_zero_stays() {
        let mut ws = dummy_workspace();
        ws.focused = 0;
        ws.focus_prev();
        assert_eq!(ws.focused, 0);
    }

    #[test]
    fn focus_prev_decrements() {
        let mut ws = dummy_workspace();
        ws.focused = 2;
        ws.focus_prev();
        assert_eq!(ws.focused, 1);
    }

    #[test]
    fn focus_next_at_end_stays() {
        let mut ws = dummy_workspace();
        ws.focused = 0;
        ws.focus_next();
        assert_eq!(ws.focused, 0);
    }

    #[test]
    fn close_on_empty_is_noop() {
        let mut ws = dummy_workspace();
        ws.close_focused_pane();
        assert!(ws.is_empty());
        assert_eq!(ws.focused, 0);
    }

    #[test]
    fn empty_workspace() {
        let ws = dummy_workspace();
        assert!(ws.is_empty());
    }

    #[test]
    fn workspace_name() {
        let ws = Workspace::with_panes("my-ws", SplitDirection::Horizontal, Vec::new());
        assert_eq!(ws.name, "my-ws");
        assert_eq!(ws.split, SplitDirection::Horizontal);
    }
}

```

