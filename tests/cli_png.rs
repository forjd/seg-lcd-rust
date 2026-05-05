use std::{fs, process::Command};

#[test]
fn cli_writes_native_png_output() {
    let output_path =
        std::env::temp_dir().join(format!("seg-lcd-rust-{}-display.png", std::process::id()));
    let _ = fs::remove_file(&output_path);

    let output = Command::new(env!("CARGO_BIN_EXE_seg-lcd-rust"))
        .args([
            "--png",
            output_path.to_str().expect("temp path should be UTF-8"),
            "--theme",
            "blue",
            "--glow",
            "12:34.5",
        ])
        .output()
        .expect("seg-lcd-rust CLI should run");

    assert!(
        output.status.success(),
        "CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let png = fs::read(&output_path).expect("PNG output should be written");
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert!(png.len() > 100);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wrote"));
    assert!(stdout.contains(output_path.to_str().unwrap()));

    let _ = fs::remove_file(output_path);
}

#[test]
fn cli_png_requires_an_output_path() {
    let output = Command::new(env!("CARGO_BIN_EXE_seg-lcd-rust"))
        .arg("--png")
        .output()
        .expect("seg-lcd-rust CLI should run");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--png requires an output path"));
}
