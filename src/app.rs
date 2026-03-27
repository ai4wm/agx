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
