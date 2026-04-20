use serde::Deserialize;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub struct Config {
	pub compiler_path: String,
	pub clang_args: Vec<String>,
	pub supported_directives: Vec<String>,
	pub default_out_name: String,
}

fn string_vec(items: &[&str]) -> Vec<String> {
	items.iter().map(|item| (*item).to_owned()).collect()
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
	compiler_path: Option<String>,
	clang_args: Option<Vec<String>>,
	supported_directives: Option<Vec<String>>,
	default_out_name: Option<String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			compiler_path: "clang".into(),
			clang_args: string_vec(&[
				"-S",
				"-target",
				"riscv32",
				"-march=rv32imfd",
				"-mabi=ilp32d",
				"-g0",
				"-mno-relax",
			]),
			supported_directives: string_vec(&[
				".text",
				".data",
				".globl",
				".type",
				".size",
				".section",
				".align",
				".p2align",
			]),
			default_out_name: "out.s".into(),
		}
	}
}

impl Config {
	pub fn load() -> Result<Self, Box<dyn Error>> {
		let mut config = Self::default();
		if let Some(path) = xdg_config_candidates()
			.into_iter()
			.find(|candidate| candidate.is_file())
		{
			let raw = fs::read_to_string(path)?;
			let overlay: PartialConfig = toml::from_str(&raw)?;
			apply_overlay(&mut config, overlay);
		}
		Ok(config)
	}
}

fn xdg_config_candidates() -> Vec<PathBuf> {
	let mut candidates = Vec::new();

	if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
		if !config_home.is_empty() {
			candidates.push(PathBuf::from(config_home).join("asm-generator/config.toml"));
		}
	} else if let Ok(home) = env::var("HOME") {
		candidates.push(PathBuf::from(home).join(".config/asm-generator/config.toml"));
	}

	let config_dirs = env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".to_string());
	for dir in config_dirs.split(':').filter(|part| !part.is_empty()) {
		candidates.push(PathBuf::from(dir).join("asm-generator/config.toml"));
	}

	candidates
}

fn apply_overlay(config: &mut Config, overlay: PartialConfig) {
	if let Some(value) = overlay.compiler_path {
		config.compiler_path = value;
	}
	if let Some(value) = overlay.clang_args {
		config.clang_args = value;
	}
	if let Some(value) = overlay.supported_directives {
		config.supported_directives = value;
	}
	if let Some(value) = overlay.default_out_name {
		config.default_out_name = value;
	}
}


