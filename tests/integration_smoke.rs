use std::process::{Command, Stdio};
use std::path::PathBuf;

#[test]
#[ignore]
fn smoke_integration_test() {
    // Locate the script
    let mut script = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    script.push("test-env/integration/smoke_test.sh");

    let status = Command::new("bash")
        .arg(script)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to spawn smoke test script");

    assert!(status.success(), "Smoke test script failed: {:?}", status);
}
