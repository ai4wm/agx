mod common;

use std::time::Duration;

use agx::agent::detector::{detect_state, AgentState};
use agx::agent::process::{AgentProcess, DEFAULT_PTY_SIZE};

#[test]
fn full_lifecycle() {
    let mut process = AgentProcess::spawn(&common::interactive_shell_command(), DEFAULT_PTY_SIZE)
        .expect("spawn shell");

    process
        .write_all(b"echo agx-lifecycle\r")
        .expect("write echo");
    let output = common::wait_for_output(
        || process.try_read(),
        "agx-lifecycle",
        Duration::from_secs(5),
    )
    .expect("output");

    assert_eq!(
        detect_state(&output, Some("agx-lifecycle"), false),
        AgentState::Idle
    );

    process.kill().expect("kill shell");
}

#[test]
fn crash_recovery() {
    let result = AgentProcess::spawn(
        &["definitely-not-a-real-command".to_string()],
        DEFAULT_PTY_SIZE,
    );

    assert!(result.is_err());
}
