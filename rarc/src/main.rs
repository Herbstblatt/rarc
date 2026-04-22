use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::process::Command;
use tempfile::tempdir;

mod args;
mod asm;
mod config;
mod process_file;
use args::rewrite_output_args;
use config::Config;
use process_file::process_file;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::load()?;
    let dir = tempdir()?;

    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    let (clang_args, rarc_args) = rewrite_output_args(cli_args, dir.path(), &config)?;

    if rarc_args.verbose {
        println!("Running clang {} {}", &clang_args.join(" "), &config.clang_args.join(" "))
    }
    let status = Command::new(&config.compiler_path)
        .args(&clang_args)
        .args(&config.clang_args)
        .status()
        .expect("Failed to launch clang compiler");

    if !status.success() {
        return Err("Compilation failed".into());
    }

    let generated_file = File::open(dir.path().join(&rarc_args.out_path))?;
    let mut reader = BufReader::new(generated_file);

    let target_file = File::create(&rarc_args.out_path)?;
    let mut writer = BufWriter::new(target_file);

    if rarc_args.output_exact {
        io::copy(&mut reader, &mut writer)?;
    } else {
        process_file(&mut reader, &mut writer, &config.supported_directives)?;
    }

    Ok(())
}
