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
