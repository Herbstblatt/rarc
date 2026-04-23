use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use wait_timeout::ChildExt;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CaseMode {
    Asm,
    Assemble,
    Run,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum WorkDirPolicy {
    Temp,
    Keep,
}

#[derive(Debug, Clone)]
struct HarnessConfig {
    rarc_bin: String,
    java_bin: String,
    rars_jar: Option<PathBuf>,
    rarc_xdg_config_home: Option<PathBuf>,
    work_dir: WorkDirPolicy,
    update_snapshots: bool,
    default_timeout_ms: u64,
}

#[derive(Debug, Default, Deserialize)]
struct HarnessConfigFile {
    rarc_bin: Option<String>,
    java_bin: Option<String>,
    rars_jar: Option<String>,
    rarc_xdg_config_home: Option<String>,
    work_dir: Option<WorkDirPolicy>,
    update_snapshots: Option<bool>,
    default_timeout_ms: Option<u64>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct CaseArgs {
    #[serde(default)]
    rarc: Vec<String>,
    #[serde(default)]
    rars: Vec<String>,
    #[serde(default)]
    clang: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
struct AsmChecks {
    #[serde(default)]
    must_contain: Vec<String>,
    #[serde(default)]
    must_not_contain: Vec<String>,
    ignore_ident: Option<bool>,
    ignore_comments: Option<bool>,
    strict_compare: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct CaseFile {
    mode: CaseMode,
    input: Option<String>,
    expected_asm: Option<String>,
    expected_stdout: Option<String>,
    expected_exit_code: Option<i32>,
    expected_success: Option<bool>,
    timeout_ms: Option<u64>,
    #[serde(default)]
    args: CaseArgs,
    #[serde(default)]
    asm: AsmChecks,
}

#[derive(Debug, Clone)]
struct Case {
    name: String,
    mode: CaseMode,
    input: PathBuf,
    expected_asm: PathBuf,
    expected_stdout: PathBuf,
    expected_exit_code: Option<i32>,
    expected_success: bool,
    timeout_ms: Option<u64>,
    args: CaseArgs,
    asm: AsmChecks,
}

#[derive(Debug)]
struct CommandOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

enum CaseOutcome {
    Passed,
    Skipped(String),
}

enum WorkDir {
    Temp(TempDir),
    Keep(PathBuf),
}

#[derive(Serialize)]
struct RarcConfigOverride {
    clang_args: Vec<String>,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            rarc_bin: "rarc".to_string(),
            java_bin: "java".to_string(),
            rars_jar: None,
            rarc_xdg_config_home: None,
            work_dir: WorkDirPolicy::Temp,
            update_snapshots: false,
            default_timeout_ms: 20_000,
        }
    }
}

impl HarnessConfig {
    fn load(manifest_dir: &Path) -> Result<Self> {
        let mut cfg = Self::default();
        let config_path = env::var("RARC_TEST_CONFIG")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                let default_path = manifest_dir.join("config.toml");
                default_path.is_file().then_some(default_path)
            });

        if let Some(path) = config_path {
            let config_base_dir = path.parent().unwrap_or(manifest_dir);
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read test config {}", path.display()))?;
            let parsed: HarnessConfigFile = toml::from_str(&raw)
                .with_context(|| format!("invalid TOML in {}", path.display()))?;
            if let Some(value) = parsed.rarc_bin {
                cfg.rarc_bin = value;
            }
            if let Some(value) = parsed.java_bin {
                cfg.java_bin = value;
            }
            if let Some(value) = parsed.rars_jar {
                cfg.rars_jar = Some(resolve_config_path(config_base_dir, &value));
            }
            if let Some(value) = parsed.rarc_xdg_config_home {
                cfg.rarc_xdg_config_home = Some(resolve_config_path(config_base_dir, &value));
            }
            if let Some(value) = parsed.work_dir {
                cfg.work_dir = value;
            }
            if let Some(value) = parsed.update_snapshots {
                cfg.update_snapshots = value;
            }
            if let Some(value) = parsed.default_timeout_ms {
                cfg.default_timeout_ms = value;
            }
        }

        if let Ok(value) = env::var("RARC_TEST_RARC_BIN") {
            if !value.is_empty() {
                cfg.rarc_bin = value;
            }
        }
        if let Ok(value) = env::var("RARC_TEST_JAVA_BIN") {
            if !value.is_empty() {
                cfg.java_bin = value;
            }
        }
        if let Ok(value) = env::var("RARC_TEST_RARS_JAR") {
            if !value.is_empty() {
                cfg.rars_jar = Some(PathBuf::from(value));
            }
        }
        if let Ok(value) = env::var("RARC_TEST_RARC_XDG_CONFIG_HOME") {
            if !value.is_empty() {
                cfg.rarc_xdg_config_home = Some(PathBuf::from(value));
            }
        }

        Ok(cfg)
    }
}

impl WorkDir {
    fn create(policy: &WorkDirPolicy, root: &Path, case_name: &str) -> Result<Self> {
        match policy {
            WorkDirPolicy::Temp => {
                let dir = tempfile::tempdir().context("failed to create temp directory")?;
                Ok(Self::Temp(dir))
            }
            WorkDirPolicy::Keep => {
                let keep_root = root.join("target").join("integration-workdir");
                fs::create_dir_all(&keep_root)
                    .with_context(|| format!("failed to create {}", keep_root.display()))?;
                let case_dir = keep_root.join(sanitize_case_name(case_name));
                if case_dir.exists() {
                    fs::remove_dir_all(&case_dir).with_context(|| {
                        format!("failed to clear old work dir {}", case_dir.display())
                    })?;
                }
                fs::create_dir_all(&case_dir)
                    .with_context(|| format!("failed to create {}", case_dir.display()))?;
                Ok(Self::Keep(case_dir))
            }
        }
    }

    fn path(&self) -> &Path {
        match self {
            WorkDir::Temp(dir) => dir.path(),
            WorkDir::Keep(path) => path,
        }
    }
}

#[test]
fn fixture_cases() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .ok_or_else(|| anyhow!("failed to determine workspace root from {}", manifest_dir.display()))?
        .to_path_buf();
    let fixtures_root = manifest_dir.join("fixtures").join("cases");
    let cfg = HarnessConfig::load(&manifest_dir)?;
    let cases = discover_cases(&fixtures_root)?;

    if cases.is_empty() {
        bail!("no fixture cases found in {}", fixtures_root.display());
    }

    let mut failures = Vec::new();
    let mut skipped = 0usize;

    for case in cases {
        match run_case(&cfg, &workspace_root, &case) {
            Ok(CaseOutcome::Passed) => {}
            Ok(CaseOutcome::Skipped(reason)) => {
                skipped += 1;
                eprintln!("[SKIP] {}: {}", case.name, reason);
            }
            Err(err) => {
                failures.push(format!("[FAIL] {}: {err:#}", case.name));
            }
        }
    }

    if !failures.is_empty() {
        let mut message = failures.join("\n");
        if skipped > 0 {
            message.push_str(&format!("\n(skipped {skipped} cases)"));
        }
        bail!(message);
    }

    if skipped > 0 {
        eprintln!("all runnable cases passed; skipped {skipped} case(s)");
    }

    Ok(())
}

fn discover_cases(fixtures_root: &Path) -> Result<Vec<Case>> {
    let mut entries = fs::read_dir(fixtures_root)
        .with_context(|| format!("failed to read {}", fixtures_root.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("failed to enumerate {}", fixtures_root.display()))?;

    entries.sort_by_key(|entry| entry.file_name());

    let mut cases = Vec::new();
    for entry in entries {
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", entry.path().display()))?;
        if !file_type.is_dir() {
            continue;
        }

        let dir = entry.path();
        let case_name = entry.file_name().to_string_lossy().to_string();
        let case_toml_path = dir.join("case.toml");
        let raw = fs::read_to_string(&case_toml_path)
            .with_context(|| format!("failed to read {}", case_toml_path.display()))?;
        let parsed: CaseFile = toml::from_str(&raw)
            .with_context(|| format!("invalid TOML in {}", case_toml_path.display()))?;

        let input = dir.join(parsed.input.unwrap_or_else(|| "input.c".to_string()));
        let expected_asm = dir.join(parsed.expected_asm.unwrap_or_else(|| "expected.s".to_string()));
        let expected_stdout =
            dir.join(parsed.expected_stdout.unwrap_or_else(|| "expected.stdout".to_string()));

        cases.push(Case {
            name: case_name,
            mode: parsed.mode,
            input,
            expected_asm,
            expected_stdout,
            expected_exit_code: parsed.expected_exit_code,
            expected_success: parsed.expected_success.unwrap_or(true),
            timeout_ms: parsed.timeout_ms,
            args: parsed.args,
            asm: parsed.asm,
        });
    }

    Ok(cases)
}

fn run_case(cfg: &HarnessConfig, workspace_root: &Path, case: &Case) -> Result<CaseOutcome> {
    let timeout = Duration::from_millis(case.timeout_ms.unwrap_or(cfg.default_timeout_ms));
    let work_dir = WorkDir::create(&cfg.work_dir, workspace_root, &case.name)?;
    let temp_input = work_dir.path().join("input.c");
    let output_asm = work_dir.path().join("out.s");

    fs::copy(&case.input, &temp_input).with_context(|| {
        format!(
            "failed to copy input {} to {}",
            case.input.display(),
            temp_input.display()
        )
    })?;

    run_rarc(cfg, workspace_root, case, &temp_input, &output_asm, timeout)?;

    match case.mode {
        CaseMode::Asm => {
            assert_asm_expectations(cfg, case, &output_asm)?;
            Ok(CaseOutcome::Passed)
        }
        CaseMode::Assemble | CaseMode::Run => {
            let rars_jar = match &cfg.rars_jar {
                Some(path) => path,
                None => {
                    return Ok(CaseOutcome::Skipped(
                        "RARS jar is not configured (set RARC_TEST_RARS_JAR)".to_string(),
                    ));
                }
            };

            if !rars_jar.is_file() {
                return Ok(CaseOutcome::Skipped(format!(
                    "RARS jar does not exist: {}",
                    rars_jar.display()
                )));
            }

            let result = run_rars(cfg, case, rars_jar, &output_asm, timeout)?;
            let success = result.status.success();
            if success != case.expected_success {
                bail!(
                    "unexpected RARS success status (expected {}, got {})\nstdout:\n{}\nstderr:\n{}",
                    case.expected_success,
                    success,
                    result.stdout,
                    result.stderr
                );
            }

            if let Some(expected_code) = case.expected_exit_code {
                let got_code = result.status.code().unwrap_or(-1);
                if got_code != expected_code {
                    bail!("unexpected exit status: expected {expected_code}, got {got_code}");
                }
            }

            if matches!(case.mode, CaseMode::Run) {
                if !case.expected_stdout.is_file() {
                    bail!(
                        "run mode requires expected stdout file: {}",
                        case.expected_stdout.display()
                    );
                }
                let expected_stdout = fs::read_to_string(&case.expected_stdout).with_context(|| {
                    format!(
                        "failed to read expected stdout {}",
                        case.expected_stdout.display()
                    )
                })?;
                if normalize_output(&result.stdout) != normalize_output(&expected_stdout) {
                    bail!(
                        "stdout mismatch\nexpected:\n{}\nactual:\n{}",
                        expected_stdout,
                        result.stdout
                    );
                }
            }

            Ok(CaseOutcome::Passed)
        }
    }
}

fn run_rarc(
    cfg: &HarnessConfig,
    workspace_root: &Path,
    case: &Case,
    input: &Path,
    out: &Path,
    timeout: Duration,
) -> Result<()> {
    let xdg_home = out
        .parent()
        .ok_or_else(|| anyhow!("output path has no parent"))?
        .join("xdg_config_home");
    let xdg_dirs = out
        .parent()
        .ok_or_else(|| anyhow!("output path has no parent"))?
        .join("xdg_config_dirs");
    if let Some(source_home) = &cfg.rarc_xdg_config_home {
        if !source_home.is_dir() {
            bail!(
                "rarc_xdg_config_home is not a directory: {}",
                source_home.display()
            );
        }
        copy_dir_recursive(source_home, &xdg_home)?;
    }

    fs::create_dir_all(&xdg_home)
        .with_context(|| format!("failed to create {}", xdg_home.display()))?;
    fs::create_dir_all(&xdg_dirs)
        .with_context(|| format!("failed to create {}", xdg_dirs.display()))?;

    if !case.args.clang.is_empty() {
        let rarc_cfg_dir = xdg_home.join("rarc");
        fs::create_dir_all(&rarc_cfg_dir)
            .with_context(|| format!("failed to create {}", rarc_cfg_dir.display()))?;
        let override_cfg = RarcConfigOverride {
            clang_args: case.args.clang.clone(),
        };
        let toml_cfg = toml::to_string(&override_cfg).context("failed to build TOML override")?;
        fs::write(rarc_cfg_dir.join("config.toml"), toml_cfg)
            .with_context(|| format!("failed to write override in {}", rarc_cfg_dir.display()))?;
    }

    let local_rarc = workspace_root.join("target").join("debug").join("rarc");
    let use_default_bin = cfg.rarc_bin == "rarc";

    let mut cmd = if use_default_bin && local_rarc.is_file() {
        Command::new(local_rarc)
    } else if use_default_bin {
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd.current_dir(workspace_root);
        cargo_cmd.arg("run").arg("-p").arg("rarc").arg("--");
        cargo_cmd
    } else {
        Command::new(&cfg.rarc_bin)
    };

    cmd.current_dir(workspace_root)
        .args(&case.args.rarc)
        .arg(input)
        .arg("-o")
        .arg(out)
        .env("XDG_CONFIG_HOME", &xdg_home)
        .env("XDG_CONFIG_DIRS", &xdg_dirs);

    let result = run_command_with_timeout(&mut cmd, timeout)
        .with_context(|| format!("failed to execute rarc for {}", case.name))?;
    if !result.status.success() {
        bail!(
            "rarc failed for case {}\nstdout:\n{}\nstderr:\n{}",
            case.name,
            result.stdout,
            result.stderr
        );
    }

    Ok(())
}

fn run_rars(
    cfg: &HarnessConfig,
    case: &Case,
    rars_jar: &Path,
    output_asm: &Path,
    timeout: Duration,
) -> Result<CommandOutput> {
    let mut cmd = Command::new(&cfg.java_bin);
    cmd.arg("-jar").arg(rars_jar);

    let mut args = match case.mode {
        CaseMode::Assemble => vec!["a".to_string(), "nc".to_string()],
        CaseMode::Run => vec!["nc".to_string()],
        CaseMode::Asm => Vec::new(),
    };
    args.extend(case.args.rars.clone());

    cmd.args(args).arg(output_asm);

    run_command_with_timeout(&mut cmd, timeout)
        .with_context(|| format!("failed to execute RARS for {}", case.name))
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .with_context(|| format!("failed to create {}", target.display()))?;

    for entry in fs::read_dir(source)
        .with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", source.display()))?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", source_path.display()))?;

        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn run_command_with_timeout(cmd: &mut Command, timeout: Duration) -> Result<CommandOutput> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn child process")?;

    match child.wait_timeout(timeout).context("wait_timeout failed")? {
        Some(_) => {
            let output = child
                .wait_with_output()
                .context("failed to collect command output")?;
            Ok(CommandOutput {
                status: output.status,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
        None => {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .context("failed to collect timed-out output")?;
            bail!(
                "command timed out after {:?}\nstdout:\n{}\nstderr:\n{}",
                timeout,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}

fn assert_asm_expectations(cfg: &HarnessConfig, case: &Case, generated_path: &Path) -> Result<()> {
    let generated = fs::read_to_string(generated_path)
        .with_context(|| format!("failed to read {}", generated_path.display()))?;

    let ignore_ident = case.asm.ignore_ident.unwrap_or(true);
    let ignore_comments = case.asm.ignore_comments.unwrap_or(true);
    let normalized_generated = normalize_asm(&generated, ignore_ident, ignore_comments);

    for needle in &case.asm.must_contain {
        if !normalized_generated.contains(needle) {
            bail!(
                "asm expectation failed: expected to contain `{}`\nfrom case {}",
                needle,
                case.name
            );
        }
    }

    for needle in &case.asm.must_not_contain {
        if normalized_generated.contains(needle) {
            bail!(
                "asm expectation failed: expected NOT to contain `{}`\nfrom case {}",
                needle,
                case.name
            );
        }
    }

    if cfg.update_snapshots {
        fs::write(&case.expected_asm, format!("{}\n", normalized_generated))
            .with_context(|| format!("failed to update {}", case.expected_asm.display()))?;
    }

    if case.asm.strict_compare.unwrap_or(false) {
        if !case.expected_asm.is_file() {
            bail!(
                "strict asm compare requested, but expected file is missing: {}",
                case.expected_asm.display()
            );
        }
        let expected = fs::read_to_string(&case.expected_asm)
            .with_context(|| format!("failed to read {}", case.expected_asm.display()))?;
        let normalized_expected = normalize_asm(&expected, ignore_ident, ignore_comments);
        if normalized_generated != normalized_expected {
            bail!(
                "strict asm compare mismatch for {}\nexpected:\n{}\nactual:\n{}",
                case.name,
                normalized_expected,
                normalized_generated
            );
        }
    }

    Ok(())
}

fn normalize_asm(input: &str, ignore_ident: bool, ignore_comments: bool) -> String {
    let mut lines = Vec::new();

    for raw_line in input.lines() {
        let trimmed_end = raw_line.trim_end();
        let trimmed_start = trimmed_end.trim_start();

        if ignore_ident && trimmed_start.starts_with(".ident") {
            continue;
        }
        if ignore_comments && trimmed_start.starts_with('#') {
            continue;
        }

        lines.push(trimmed_end.to_string());
    }

    lines.join("\n")
}

fn normalize_output(input: &str) -> String {
    input.replace("\r\n", "\n")
}

fn sanitize_case_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn resolve_config_path(base_dir: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}
