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
