mod common;

use agx::agent::registry::AgentRegistry;
use agx::config::loader::Config;

#[test]
fn config_to_registry() {
    let (_dir, path) = common::temp_config(
        r#"
[keybind]
prefix = "Ctrl-a"

[defaults]
shell = "powershell.exe"
split = "vertical"

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
    );

    let config = Config::load_from_path(&path).expect("load config");
    let mut registry = AgentRegistry::new();
    for agent in config.agent_definitions() {
        registry.register(agent).expect("register agent");
    }

    let claude = registry.get("claude").expect("claude registered");
    let codex = registry.get("codex").expect("codex registered");

    assert_eq!(claude.command, "claude");
    assert_eq!(codex.command, "codex");
}
