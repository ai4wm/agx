# GPT-5.4 ?꾨떖???꾨＼?꾪듃

```markdown
# agx v0.3 ?ъ꽕怨?- cmux 怨꾩링 + Alt ?ㅻ컮?몃뵫 + ?ъ씠?쒕컮 ?ъ빱??
## 湲닿툒: 2媛吏 ?洹쒕え 蹂寃?1. 怨꾩링 援ъ“瑜?cmux? ?숈씪?섍쾶 蹂寃? Workspace -> Pane(遺꾪븷) -> Surface(??
2. prefix 紐⑤뱶 ?꾩쟾 ?쒓굅. Alt ??湲곕컲 吏곸젒 諛붿씤?⑹쑝濡??꾨㈃ 援먯껜.

?댁쟾 v0.3 ?묒뾽(Tab 紐⑤뜽, prefix ????**?꾨? 援먯껜**?⑸땲??

## cmux 怨꾩링 (?닿쾬??洹몃?濡??곕쫫)
```text
Workspace (?ъ씠?쒕컮 ??ぉ)
   Pane (遺꾪븷 ?곸뿭)
         Surface (?⑥씤 ?덉쓽 ??
               Terminal (PTY ?꾨줈?몄뒪)
```

## cmux ?쒓컖 援ъ“
```text
 Sidebar   Pane 1            Pane 2
           [S1] [S2] [S3]   [S1] [S2]
 > dev
   server
   logs     Terminal          Terminal

 agx  ws:dev  panes:2  Alt+?:help
```

?듭떖: 媛?Pane(遺꾪븷 ?곸뿭) ?덉뿉 ?먯껜 ??컮媛 ?덇퀬, 洹???씠 Surface?낅땲??

## ?ъ빱??紐⑤뜽

agx?먮뒗 ??媛吏 ?ъ빱???곸뿭???덉뒿?덈떎:

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FocusArea {
    Sidebar,
    Panes,
}
```

App??`pub focus: FocusArea` ?꾨뱶瑜?異붽??⑸땲?? 湲곕낯媛믪? `FocusArea::Panes`?낅땲??
App??`pub sidebar_cursor: usize` ?꾨뱶瑜?異붽??⑸땲?? ?ъ씠?쒕컮 ?ъ빱??吏꾩엯 ???꾩옱 ?뚰겕?ㅽ럹?댁뒪 ?몃뜳?ㅻ줈 珥덇린?뷀빀?덈떎.
?⑥씤 ?ъ빱???곹깭?먯꽌??`current_workspace.focused_pane`媛 ?ㅼ젣 ?ъ빱?ㅻ맂 ?⑥씤???섑??낅땲??

## Tab ???숈옉
- Tab: ?ъ빱?ㅻ? ?쒗솚 ?대룞 (?ъ씠?쒕컮 -> Pane1 -> Pane2 -> ... -> ?ъ씠?쒕컮)
- Shift+Tab: ??갑???쒗솚
- ?ъ씠?쒕컮 ?ъ빱?ㅼ씪 ?? ?뚰겕?ㅽ럹?댁뒪 ?좏깮, Enter ?꾪솚
- ?⑥씤 ?ъ빱?ㅼ씪 ?? Tab ?쒖쇅??紐⑤뱺 ?ㅻ뒗 ?먯씠?꾪듃???꾨떖

### ?ъ빱???꾪솚
- Tab: ?ъ씠?쒕컮? 媛??⑥씤???쒗솚 ?대룞
- Shift+Tab: 諛섎? 諛⑺뼢 ?쒗솚
- Alt+B: ?ъ씠?쒕컮 ?닿린+?ъ빱??/ ?リ린
- Enter (?ъ씠?쒕컮?먯꽌): ?좏깮???뚰겕?ㅽ럹?댁뒪濡??꾪솚 + Panes濡??ъ빱???대룞
- Esc (?ъ씠?쒕컮?먯꽌): Panes濡??ъ빱???대룞

### ???낅젰 ?쇱슦??
```rust
fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
    // 1. Alt 議고빀? ?대뵒?쒕뱺 agx媛 泥섎━
    if key.modifiers.contains(KeyModifiers::ALT) {
        return self.handle_alt_key(key);
    }

    // 2. Tab / Shift+Tab ? ?ъ빱???쒗솚
    if key.code == KeyCode::Tab && key.modifiers.is_empty() {
        self.cycle_focus_forward();
        return Ok(());
    }
    if key.code == KeyCode::BackTab {
        self.cycle_focus_backward();
        return Ok(());
    }

    // 3. ?ъ빱???곸뿭???곕씪 遺꾧린
    match self.focus {
        FocusArea::Sidebar => self.handle_sidebar_key(key),
        FocusArea::Panes => {
            if let Some(ws) = self.current_workspace_mut() {
                ws.send_key_to_focused(key);
            }
            Ok(())
        }
    }
}
```

### ?ъ빱???쒗솚 ?덉떆

```rust
fn cycle_focus_forward(&mut self) {
    match self.focus {
        FocusArea::Sidebar => {
            self.focus = FocusArea::Panes;
            if let Some(ws) = self.current_workspace_mut() {
                ws.focused_pane = 0;
            }
        }
        FocusArea::Panes => {
            if let Some(ws) = self.current_workspace_mut() {
                if ws.focused_pane + 1 < ws.panes.len() {
                    ws.focused_pane += 1;
                } else {
                    self.focus = FocusArea::Sidebar;
                    self.sidebar_cursor = self.current_workspace;
                }
            }
        }
    }
}

fn cycle_focus_backward(&mut self) {
    match self.focus {
        FocusArea::Sidebar => {
            self.focus = FocusArea::Panes;
            if let Some(ws) = self.current_workspace_mut() {
                ws.focused_pane = ws.panes.len().saturating_sub(1);
            }
        }
        FocusArea::Panes => {
            if let Some(ws) = self.current_workspace_mut() {
                if ws.focused_pane > 0 {
                    ws.focused_pane -= 1;
                } else {
                    self.focus = FocusArea::Sidebar;
                    self.sidebar_cursor = self.current_workspace;
                }
            }
        }
    }
}
```

### ?ъ씠?쒕컮 ??泥섎━

```rust
fn handle_sidebar_key(&mut self, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Up => { /* sidebar_cursor ?꾨줈 */ }
        KeyCode::Down => { /* sidebar_cursor ?꾨옒濡?*/ }
        KeyCode::Enter => {
            self.current_workspace = self.sidebar_cursor;
            self.focus = FocusArea::Panes;
        }
        KeyCode::Esc => {
            self.focus = FocusArea::Panes;
        }
        _ => {}
    }
    Ok(())
}
```

## ?ㅻ컮?몃뵫 ?꾩껜 (理쒖쥌)

### Alt ??(?ъ빱???꾩튂 臾닿?, ??긽 ?숈옉)

| ??| ?숈옉 |
|---|---|
| Alt+D | ?ㅻⅨ履쎌쑝濡?遺꾪븷 (??Pane) |
| Alt+S | ?꾨옒濡?遺꾪븷 (??Pane) |
| Alt+T | ?꾩옱 ?⑥씤????Surface(?? |
| Alt+W | ?꾩옱 Surface ?リ린 |
| Alt+諛⑺뼢??| ?⑥씤 媛??대룞 |
| Alt+[ / Alt+] | ?⑥씤 ??Surface ?꾪솚 |
| Alt+1~9 | ?뚰겕?ㅽ럹?댁뒪 ?꾪솚 |
| Alt+C | ???뚰겕?ㅽ럹?댁뒪 |
| Alt+X | ?뚰겕?ㅽ럹?댁뒪 ?リ린 |
| Alt+B | ?ъ씠?쒕컮 ?좉? (?닿린+?ъ빱??/ ?リ린) |
| Alt+Q | agx 醫낅즺 |

### ?ъ빱???꾪솚

| ??| ?숈옉 |
|---|---|
| Tab | ?ъ씠?쒕컮 -> Pane1 -> Pane2 -> ... -> ?ъ씠?쒕컮 ?쒗솚 |
| Shift+Tab | ??갑???쒗솚 |

### ?ъ씠?쒕컮 ?ъ빱???곹깭?먯꽌

| ??| ?숈옉 |
|---|---|
| Up / Down | ?뚰겕?ㅽ럹?댁뒪 而ㅼ꽌 ?대룞 |
| Enter | ?좏깮???뚰겕?ㅽ럹?댁뒪濡??꾪솚 + Panes濡??대룞 |
| Esc | Panes濡??ъ빱???대룞 |

### ?⑥씤 ?ъ빱???곹깭?먯꽌

| ??| ?숈옉 |
|---|---|
| 紐⑤뱺 ??(Alt/Tab ?쒖쇅) | ?ъ빱?ㅻ맂 ?⑥씤???꾩옱 Surface(PTY)???꾨떖 |

## ?뚯씪 援ъ“ 蹂寃?
### ??젣
- src/tab.rs

### ?덈줈 ?앹꽦
- src/surface.rs
- src/pane.rs

### ?섏젙
- src/workspace.rs
- src/app.rs
- src/config.rs (?먮뒗 config/loader.rs) prefix 愿???ㅼ젙 ?쒓굅, keybind ?뱀뀡 ?⑥닚??- src/ui/mod.rs
- src/ui/layout.rs
- src/ui/sidebar.rs
- src/agent/mod.rs 蹂寃?理쒖냼?? AgentPane 洹몃?濡??좎?
- src/lib.rs
- tests/ 愿???뚯뒪??媛깆떊

### ?좎? (蹂寃??놁쓬)
- src/terminal/pty.rs
- src/terminal/mod.rs

## 紐⑤뜽 ?ㅺ퀎

### src/surface.rs
```rust
use crate::agent::AgentPane;
use crate::agent::PaneSpec;
use anyhow::Result;

/// Surface = ?⑥씤 ?덉쓽 ?? cmux??Surface? ?숈씪.
pub struct Surface {
    pub label: String,
    pub agent: AgentPane,
}

impl Surface {
    pub fn new(spec: PaneSpec) -> Result<Self> {
        let agent = AgentPane::spawn(spec.clone())?;
        Ok(Self {
            label: spec.label,
            agent,
        })
    }

    pub fn poll(&mut self) {
        self.agent.poll();
    }

    pub fn is_exited(&self) -> bool {
        self.agent.exited
    }
}
```

### src/pane.rs
```rust
use crate::surface::Surface;
use crate::agent::PaneSpec;
use anyhow::Result;

/// Pane = 遺꾪븷 ?곸뿭. cmux??Pane怨??숈씪.
/// ?щ윭 Surface(??瑜?媛吏硫? ?먯껜 ??컮瑜?媛吏?
pub struct Pane {
    pub surfaces: Vec<Surface>,
    pub current_surface: usize,
}

impl Pane {
    pub fn new(spec: PaneSpec) -> Result<Self> {
        let surface = Surface::new(spec)?;
        Ok(Self {
            surfaces: vec![surface],
            current_surface: 0,
        })
    }

    pub fn add_surface(&mut self, spec: PaneSpec) -> Result<()>;
    pub fn close_current_surface(&mut self);
    pub fn next_surface(&mut self);
    pub fn prev_surface(&mut self);
    pub fn current_surface(&self) -> Option<&Surface>;
    pub fn current_surface_mut(&mut self) -> Option<&mut Surface>;
    pub fn poll(&mut self);
    pub fn is_empty(&self) -> bool;
    pub fn send_key(&mut self, key: KeyEvent);
    pub fn resize(&mut self, rows: u16, cols: u16);
}
```

### src/workspace.rs
```rust
use crate::pane::Pane;
use crate::agent::PaneSpec;
use crate::SplitDirection;
use anyhow::Result;

/// Workspace = ?ъ씠?쒕컮 ??ぉ. cmux??Workspace? ?숈씪.
pub struct Workspace {
    pub name: String,
    pub panes: Vec<Pane>,
    pub focused_pane: usize,
    pub split: SplitDirection,
}

impl Workspace {
    pub fn new(name: impl Into<String>, spec: PaneSpec, split: SplitDirection) -> Result<Self>;
    pub fn split_right(&mut self, spec: PaneSpec) -> Result<()>;
    pub fn split_down(&mut self, spec: PaneSpec) -> Result<()>;
    pub fn close_focused_pane(&mut self);
    pub fn focus_left(&mut self);
    pub fn focus_right(&mut self);
    pub fn focus_up(&mut self);
    pub fn focus_down(&mut self);
    pub fn add_surface_to_focused(&mut self, spec: PaneSpec) -> Result<()>;
    pub fn close_current_surface(&mut self);
    pub fn next_surface(&mut self);
    pub fn prev_surface(&mut self);
    pub fn send_key_to_focused(&mut self, key: KeyEvent);
    pub fn poll(&mut self);
    pub fn is_empty(&self) -> bool;
}
```

### src/app.rs handle_key 蹂寃?```rust
fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
    if key.modifiers.contains(KeyModifiers::ALT) {
        return self.handle_alt_key(key);
    }

    if key.code == KeyCode::Tab && key.modifiers.is_empty() {
        self.cycle_focus_forward();
        return Ok(());
    }
    if key.code == KeyCode::BackTab {
        self.cycle_focus_backward();
        return Ok(());
    }

    match self.focus {
        FocusArea::Sidebar => self.handle_sidebar_key(key),
        FocusArea::Panes => {
            if let Some(workspace) = self.current_workspace_mut() {
                workspace.send_key_to_focused(key);
            }
            Ok(())
        }
    }
}
```

## config.toml 蹂寃?
prefix 愿???ㅼ젙 ?쒓굅. keybind ?뱀뀡 ?⑥닚??

```toml
# config.toml
[defaults]
shell = "powershell.exe"
split = "vertical"

[[agent]]
name = "claude"
command = "claude"
detect_idle = ""
color = "cyan"
```

[keybind] ?뱀뀡??prefix ?꾨뱶 ?쒓굅. KeyBinding 援ъ“泥댁뿉??prefix 愿???뚯떛/留ㅼ묶 濡쒖쭅 ?쒓굅 ?먮뒗 ?ъ슜?섏? ?딅룄濡?蹂寃? config.rs?먯꽌 prefix_binding() 硫붿꽌???쒓굅.

## ?덉씠?꾩썐 怨꾩궛

```text
?꾩껜 ?곸뿭
 ?ъ씠?쒕컮: 怨좎젙 18 而щ읆 (?좉? 媛??
 硫붿씤 ?곸뿭
     ?⑥씤 遺꾪븷 (Workspace.split 湲곗?)
        Pane 1 (蹂대뜑 ?ы븿 ?꾩껜)
           ??컮: 1以?([S1] [S2] ...)   Surface 2媛??댁긽???뚮쭔 ?쒖떆
           ?곕????곸뿭: ?섎㉧吏
        Pane 2 (蹂대뜑 ?ы븿 ?꾩껜)
           ??컮: 1以?           ?곕????곸뿭: ?섎㉧吏
     ?곹깭諛? 1以?```

```rust
pub struct LayoutState {
    pub sidebar: Option<Rect>,
    pub pane_layouts: Vec<PaneLayout>,
    pub status_area: Rect,
}

pub struct PaneLayout {
    pub outer: Rect,
    pub tabbar: Option<Rect>,
    pub content: Rect,
}
```

## ?ъ씠?쒕컮 UI ?뚮뜑留?蹂寃?
?ъ씠?쒕컮 ?ъ빱???곹깭???곕씪 ?쒓컖??援щ텇:

```text
?ъ빱???덉쓣 ??
  Workspaces
    ws1
  > ws2      <- 而ㅼ꽌 ?꾩튂
    ws3

  蹂대뜑 ?됱긽: Cyan

?ъ빱???놁쓣 ??
  Workspaces
    ws1
  * ws2      <- ?꾩옱 ?뚰겕?ㅽ럹?댁뒪
    ws3

  蹂대뜑 ?됱긽: DarkGray
```

## UI ?뚮뜑留??쒖꽌
1. ?ъ씠?쒕컮 (?뚰겕?ㅽ럹?댁뒪 紐⑸줉, ?꾩옱 ?좏깮 ?섏씠?쇱씠?? ?ъ빱????而ㅼ꽌 媛뺤“)
2. 媛?Pane留덈떎:
   a. 蹂대뜑 + ?쒕ぉ (?ъ빱?ㅻ㈃ Cyan, ?꾨땲硫?DarkGray)
   b. Surface 2媛??댁긽?대㈃ ?⑥씤 ?대? ?곷떒????컮 ?뚮뜑留?   c. ?꾩옱 Surface??vt100 ?ㅽ겕由??뚮뜑留?3. ?곹깭諛? agx 濡쒓퀬 + ?꾩옱 ?뚰겕?ㅽ럹?댁뒪 + pane ??+ "Alt+?:help"

## ?뚯뒪???붽뎄?ы빆

### ???뚯뒪??- surface.rs: ?앹꽦 (PTY ?꾩슂?섎?濡?#[ignore] 媛??
- pane.rs: add/close/next/prev surface 寃쎄퀎媛?(PTY ?놁씠 媛?ν븳 遺遺?
- workspace.rs: split_right/down, focus ?대룞, close_focused_pane, is_empty
- ui/layout.rs: ?ъ씠?쒕컮 ?덉쓣 ???놁쓣 ?? ?⑥씤 1~4媛? PaneLayout tabbar/content 遺꾨━
- app.rs: handle_alt_key 遺꾧린 ?뚯뒪??- handle_sidebar_key: Up/Down 而ㅼ꽌 ?대룞 寃쎄퀎媛?- handle_sidebar_key: Enter -> ?뚰겕?ㅽ럹?댁뒪 ?꾪솚 + ?ъ빱???대룞
- handle_sidebar_key: Esc -> ?⑥씤 ?ъ빱??蹂듦?
- cycle_focus_forward / cycle_focus_backward: ?ъ씠?쒕컮 <-> Pane1..N ?쒗솚
- sidebar_cursor 珥덇린?? ?ъ씠?쒕컮 吏꾩엯 ???꾩옱 ?뚰겕?ㅽ럹?댁뒪 ?몃뜳??
### ?좎?
- config ?뚯뒪?? prefix 愿???뚯뒪???쒓굅/?섏젙, ?섎㉧吏 ?좎?
- agent/mod.rs: encode_key, PaneSpec, idle detection ?뚯뒪??洹몃?濡?- 珥??뚯뒪????84媛??댁긽 ?좎?

### 寃利?湲곗?
- cargo test ?듦낵
- cargo clippy --workspace -- -D warnings ?듦낵
- cargo build ?듦낵

## ?쒖빟?ы빆
- cmux 肄붾뱶(AGPL) 吏곸젒 李⑥슜 湲덉?. 援ъ“/?명꽣?섏씠?ㅻ쭔 李멸퀬, clean-room 援ы쁽
- ?щ줈?ㅽ뵆?ロ뤌 ?좎? (Windows ConPTY + Unix PTY)
- AgentPane(PTY ?섑띁), terminal/pty.rs??蹂寃쏀븯吏 ?딆쓬
- ?먮윭 泥섎━: anyhow
- edition = "2021"

## 異쒕젰 ?뺤떇
- 蹂寃?異붽?/??젣?섎뒗 ?뚯씪蹂꾨줈 ?꾩껜 肄붾뱶 ?쒓났
- ?뚯씪 寃쎈줈 紐낇솗???쒓린
- PowerShell Set-Content 紐낅졊?대줈 諛붾줈 遺숈뿬?ｊ린 媛?ν븳 ?뺥깭
```

## ?꾩옱 src ?꾩껜 ?뚯씪 ?댁슜
?꾨옒?????묒뾽怨듦컙???꾩옱 `src/` ?꾩껜 ?뚯씪 ?꾨Ц?낅땲?? ?대쾲 援먯껜 ?묒뾽?먯꽌???뱁엳 `src/app.rs`, `src/workspace.rs`, `src/pane.rs`, `src/surface.rs`, `src/ui/mod.rs`, `src/ui/layout.rs`, `src/ui/sidebar.rs`, `src/terminal/input.rs`???꾩옱 ?곹깭瑜??뺥솗??李멸퀬?댁꽌 ?泥댄빐???⑸땲??
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
            encode_key(key(KeyCode::Char('??), KeyModifiers::empty())),
            "??.as_bytes().to_vec()
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
use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
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
            AppCommand::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            AppCommand::NewWorkspace => self.create_workspace()?,
            AppCommand::CloseWorkspace => self.close_current_workspace(),
            AppCommand::SwitchWorkspace(index) => {
                if index < self.workspaces.len() {
                    self.current_workspace = index;
                }
            }
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
        Ok(())
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

    fn empty_app() -> App {
        App {
            workspaces: vec![empty_workspace("ws1")],
            current_workspace: 0,
            should_quit: false,
            show_sidebar: true,
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

    fn alt_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
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

## src\lib.rs
```rust
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

## src\pane.rs
```rust
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

## src\surface.rs
```rust
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

## src\terminal\input.rs
```rust
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
    Quit,
}

pub fn command_for_key(key: KeyEvent) -> Option<AppCommand> {
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

## src\ui\mod.rs
```rust
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

## src\ui\sidebar.rs
```rust
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::workspace::Workspace;

pub const SIDEBAR_WIDTH: u16 = 18;

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkspaceEntry {
    name: String,
    pane_count: usize,
    is_current: bool,
}

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let entries = workspace_entries(&app.workspaces, app.current_workspace);
    let mut lines = Vec::new();

    for entry in entries {
        let marker = if entry.is_current { ">" } else { " " };
        let name_style = if entry.is_current {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![Span::styled(
            format!("{marker}{}", entry.name),
            name_style,
        )]));
        lines.push(Line::from(vec![Span::styled(
            format!(" panes:{}", entry.pane_count),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            " no workspaces",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let sidebar = Paragraph::new(lines).block(
        Block::default()
            .title(" ws ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(sidebar, area);
}

fn workspace_entries(workspaces: &[Workspace], current_workspace: usize) -> Vec<WorkspaceEntry> {
    workspaces
        .iter()
        .enumerate()
        .map(|(index, workspace)| WorkspaceEntry {
            name: workspace.name.clone(),
            pane_count: workspace.panes.len(),
            is_current: index == current_workspace,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{workspace_entries, SIDEBAR_WIDTH};
    use crate::workspace::Workspace;
    use crate::SplitDirection;

    #[test]
    fn sidebar_width_matches_spec() {
        assert_eq!(SIDEBAR_WIDTH, 18);
    }

    #[test]
    fn entries_include_pane_count_and_selection() {
        let workspaces = vec![
            Workspace {
                name: "ws1".to_string(),
                panes: Vec::new(),
                focused_pane: 0,
                split: SplitDirection::Vertical,
            },
            Workspace {
                name: "ws2".to_string(),
                panes: Vec::new(),
                focused_pane: 0,
                split: SplitDirection::Horizontal,
            },
        ];

        let entries = workspace_entries(&workspaces, 1);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].pane_count, 0);
        assert_eq!(entries[1].pane_count, 0);
        assert!(!entries[0].is_current);
        assert!(entries[1].is_current);
    }
}
```

## src\workspace.rs
```rust
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

