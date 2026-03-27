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
