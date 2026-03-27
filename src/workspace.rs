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
