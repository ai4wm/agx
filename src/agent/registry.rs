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
