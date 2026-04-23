use assert_cmd::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn command_exists(program: &str) -> bool {
    Command::new(program).arg("--version").output().is_ok()
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn run_rarc_on_fixture(fixture_name: &str, output_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    if !command_exists("clang") {
        eprintln!("Skipping test because clang is not available in PATH");
        return Ok(String::new());
    }

    let dir = tempdir()?;
    let output_path = dir.path().join(output_name);

    let assert = Command::cargo_bin("rarc")?
        .arg(fixture_path(fixture_name))
        .arg("-o")
        .arg(&output_path)
        .env("XDG_CONFIG_HOME", dir.path().join("xdg-config-home"))
        .env("XDG_CONFIG_DIRS", dir.path().join("xdg-config-dirs"))
        .assert();

    assert.success();
    Ok(fs::read_to_string(output_path)?)
}

#[test]
fn full_pipeline_wraps_main_function() {
    let output = run_rarc_on_fixture("wrap_main.c", "wrap_main.s")
        .expect("rarc pipeline should succeed");
    if output.is_empty() {
        return;
    }

    assert!(
        output.starts_with(
            "# This file was generated automatically by the rarc tool. If this line has any lines above, DO NOT MODIFY THEM.\n\n"
        ),
        "missing generated header"
    );
    assert!(output.contains(".globl main"));
    assert!(output.contains("main:\n"));
    assert!(output.contains("call __rarc_original_main\n"));
    assert!(output.contains(".globl __rarc_original_main"));
    assert!(output.contains("__rarc_original_main:"));
}

#[test]
fn rewrites_bss_zero_to_data_chunks() {
    let output = run_rarc_on_fixture("global_zero.c", "global_zero.s")
        .expect("rarc pipeline should succeed");
    if output.is_empty() {
        return;
    }

    assert!(output.contains("\t.data\n"), "expected .data directive in output");
    assert!(!output.contains("\t.bss\n"), "unexpected .bss directive in output");
    assert!(!output.contains("\t.zero "), "unexpected .zero directive in output");
    assert!(
        output.contains("\t.half 0\n\t.byte 0\n") || output.contains("\t.word 0\n"),
        "expected zero-fill expansion into supported data directives"
    );
}

#[test]
fn preserves_symbol_visibility_for_globals_and_statics() {
    let output = run_rarc_on_fixture("visibility.c", "visibility.s")
        .expect("rarc pipeline should succeed");
    if output.is_empty() {
        return;
    }

    assert!(output.contains("global_var:"), "global variable label should exist");
    assert!(output.contains("static_var:"), "static variable label should exist");
    assert!(output.contains("global_func:"), "global function label should exist");
    assert!(output.contains("static_func:"), "static function label should exist");

    assert!(
        output.contains(".globl global_var"),
        "global variable should be marked global"
    );
    assert!(
        output.contains(".globl global_func"),
        "global function should be marked global"
    );

    assert!(
        !output.contains(".globl static_var"),
        "static variable must not be marked global"
    );
    assert!(
        !output.contains(".globl static_func"),
        "static function must not be marked global"
    );
}
