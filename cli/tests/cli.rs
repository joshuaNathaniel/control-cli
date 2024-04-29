use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("control")?;

    cmd.args([
        "control",
        "code",
        "--lang",
        "java",
        "--ext",
        "java",
        "--output-file",
        "hashDerp",
        "./turd/mcdermott",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: No commented code found."));

    Ok(())
}

#[test]
fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("control")?;
    cmd.args([
        "control",
        "code",
        "--lang",
        "java",
        "--ext",
        "java",
        "--output-file",
        ".control-log",
        "./tests/resources/java",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(".control-log generated.\n"));

    Ok(())
}
#[test]
fn fail_unsupported_language() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("control")?;
    cmd.args([
        "control",
        "code",
        "--lang",
        "guava",
        "--ext",
        "guava",
        "--output-file",
        "hashDerp",
        "./tests/resources/java",
    ]);
    cmd.assert().failure();
    // .stderr(predicate::str::contains("Language not supported"));

    Ok(())
}
