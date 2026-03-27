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
