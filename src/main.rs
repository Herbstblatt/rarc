use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process::Command;
use tempfile::tempdir;

mod args;
mod config;
mod process_file;
use args::rewrite_output_args;
use config::Config;
use process_file::process_file;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;
    let dir = tempdir()?;

    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    let (args, out_path) = rewrite_output_args(cli_args, dir.path(), &config.default_out_name)?;

    let status = Command::new(&config.compiler_path)
        .args(&config.clang_args)
        .args(&args)
        .status()
        .expect("Failed to launch clang compiler");

    if !status.success() {
        return Err("Compilation failed".into());
    }

    let generated_file = File::open(dir.path().join(&out_path))?;
    let reader = BufReader::new(generated_file);
    let target_file = File::create(&out_path)?;
    let writer = BufWriter::new(target_file);

    process_file(reader, writer, &config.supported_directives)?;

    Ok(())
}
