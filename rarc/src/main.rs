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

const USAGE: &str = r#"Usage: rarc [clang options] [--output-exact] [-o <out-path>] [-v|--verbose]

Description:
    rarc — a small wrapper around the clang compiler. It runs clang with the
    provided options and then processes the generated object file, replacing or
    handling custom directives supported by rarc.

rarc options:
    --output-exact        Copy the generated file without processing
    -o <out-path>         Path to the resulting file
    -v, --verbose         Print additional debugging information
    -h, --help            Show this message and exit
    -V, --version         Show version and exit

Example:
    rarc src/main.c -o out.o -- -DDEBUG
"#;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn handle_global_flags(cli_args: &[String]) -> bool {
    if cli_args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", USAGE);
        return true;
    }

    if cli_args.iter().any(|a| a == "--version" || a == "-V") {
        println!("rarc {}", VERSION);
        return true;
    }

    false
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    if handle_global_flags(&cli_args) {
        return Ok(());
    }

    let config = Config::load()?;
    let dir = tempdir()?;
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

    let generated_file = File::open(&rarc_args.generated_path)?;
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
