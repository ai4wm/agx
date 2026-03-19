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
