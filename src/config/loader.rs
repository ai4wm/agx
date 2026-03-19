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
}
