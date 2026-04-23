use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::config::Config;

pub struct Args {
    pub out_path: String,
    pub generated_path: PathBuf,
    pub output_exact: bool,
    pub verbose: bool
}

pub fn rewrite_output_args(
    mut args: Vec<String>,
    base_dir: &Path,
    config: &Config,
) -> io::Result<(Vec<String>, Args)> {
    let mut out_path: Option<String> = None;
    let mut generated_path: Option<PathBuf> = None;
    let mut output_exact = false;
    let mut verbose = false;

    let mut iter = args.iter_mut();
    while let Some(item) = iter.next() {
        if item == "--output-exact" {
            output_exact = true;
            item.clear();
        } else if item == "-v" || item == "--verbose" {
            verbose = true;
        } else if item == "-o" {
            if let Some(path) = iter.next() {
                let requested_out_path = path.clone();
                out_path = Some(requested_out_path.clone());

                let temp_file_name = Path::new(&requested_out_path)
                    .file_name()
                    .map(|name| name.to_owned())
                    .unwrap_or_else(|| config.default_out_name.as_str().into());
                let rewritten_path = base_dir.join(temp_file_name);
                generated_path = Some(rewritten_path.clone());

                *path = rewritten_path
                    .to_str()
                    .expect("Path should contain valid unicode")
                    .to_string();
            } else {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "Invalid arguments: expected value after -o",
                ));
            }
        }
    }

    for path in &config.include_paths {
        args.push("-I".to_owned() + path);
    }

    let out_path = out_path.unwrap_or_else(|| {
        args.push("-o".into());
        let default_generated = base_dir.join(&config.default_out_name);
        generated_path = Some(default_generated.clone());
        args.push(default_generated.to_str().expect("Path should contain valid unicode").to_string());
        config.default_out_name.clone()
    });

    let generated_path = generated_path.expect("Generated output path should always be set");

    let rarc_args = Args {
        out_path: out_path,
        generated_path,
        output_exact: output_exact,
        verbose: verbose
    };
    Ok((args, rarc_args))
}
