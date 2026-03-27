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
