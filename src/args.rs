use std::io;
use std::io::ErrorKind;
use std::path::Path;

pub struct Args {
    pub out_path: String,
    pub output_exact: bool
}

pub fn rewrite_output_args(
    mut args: Vec<String>,
    base_dir: &Path,
    default_out_name: &str,
) -> io::Result<(Vec<String>, Args)> {
    let mut out_path: Option<String> = None;
    let mut output_exact = false;

    let mut iter = args.iter_mut();
    while let Some(item) = iter.next() {
        if item == "--output-exact" {
            output_exact = true;
            item.clear();
        } else if item == "-o" {
            if let Some(path) = iter.next() {
                out_path = Some(path.clone());
                *path = base_dir
                    .join(&*path)
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

    let out_path = out_path.unwrap_or_else(|| {
        args.push("-o".into());
        args.push(
            base_dir
                .join(default_out_name)
                .to_str()
                .expect("Path should contain valid unicode")
                .to_string(),
        );
        default_out_name.to_string()
    });

    let rarc_args = Args {
        out_path: out_path,
        output_exact: output_exact
    };
    Ok((args, rarc_args))
}
