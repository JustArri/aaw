use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    if env::args_os().len() != 2 {
        eprintln!("error: expected one argument - destination (output directory)");
        print_help();
        return ExitCode::FAILURE;
    }

    let destination = PathBuf::from(env::args_os().nth(1).unwrap());
    if !destination.is_dir() {
        eprintln!("error: destination must be a directory");
    }

    if !generate_openapi(&destination) {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn generate_openapi(destination: &Path) -> bool {
    let output = destination.join("openapi.json");
    // The DAV filesystem is not accessed while collecting OpenAPI metadata.
    let openapi = aaw::server::api_router(".").into_openapi();

    let json = match openapi.to_pretty_json() {
        Ok(json) => json,
        Err(error) => {
            eprintln!("failed to serialize OpenAPI: {error}");
            return false;
        }
    };

    if let Some(parent) = output.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("failed to create directory '{}': {error}", parent.display());
        return false;
    }

    if let Err(error) = fs::write(&output, json) {
        eprintln!("failed to write '{}': {error}", maybe_abs_path(&output).display());
        return false;
    }

    println!("generated {}", maybe_abs_path(&output).display());
    true
}

fn print_help() {
    eprintln!("Usage: generate-api <destination>");
    eprintln!();
    eprintln!("Example:");
    eprintln!("  generate-api ./generated");
}

fn maybe_abs_path(path: &Path) -> PathBuf {
    let non_canonical = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().unwrap_or(PathBuf::from(".")).join(path)
    };

    std::fs::canonicalize(&non_canonical).unwrap_or(non_canonical)
}
