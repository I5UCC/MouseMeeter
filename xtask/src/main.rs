//! `cargo xtask package [--no-build]`
//!
//! Builds the release binary and assembles `dist/windows/` plus `dist/Mousemeeter-windows.zip`.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

const EXE: &str = "Mousemeeter.exe";
const PLATFORM: &str = "windows";

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the workspace")
        .to_path_buf()
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("package") => package(&args[1..]),
        _ => {
            eprintln!("usage: cargo xtask package [--no-build]");
            std::process::exit(2);
        }
    }
}

fn package(args: &[String]) -> Result<()> {
    let mut build = true;
    for arg in args {
        match arg.as_str() {
            "--no-build" => build = false,
            other => bail!("unknown argument '{other}'"),
        }
    }
    if !cfg!(windows) {
        bail!("Mousemeeter only builds for Windows");
    }

    let root = workspace_root();
    if build {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let status = Command::new(cargo)
            .current_dir(&root)
            .args(["build", "--release", "--package", "mousemeeter"])
            .status()
            .context("failed to run cargo")?;
        if !status.success() {
            bail!("cargo build failed");
        }
    }

    let exe = root.join("target").join("release").join(EXE);
    if !exe.is_file() {
        bail!("{} not found", exe.display());
    }

    let dist = root.join("dist");
    let stage = dist.join(PLATFORM);
    if stage.exists() {
        fs::remove_dir_all(&stage)?;
    }
    fs::create_dir_all(&stage)?;

    let copy = |src: PathBuf, name: &str| -> Result<()> {
        fs::copy(&src, stage.join(name))
            .with_context(|| format!("copying {}", src.display()))
            .map(|_| ())
    };
    copy(exe, EXE)?;
    copy(root.join("LICENSE"), "LICENSE")?;

    let archive = dist.join(format!("Mousemeeter-{PLATFORM}.zip"));
    write_zip(&stage, &archive)?;
    println!("created {}", archive.display());
    Ok(())
}

fn files_in(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    files.sort();
    Ok(files)
}

fn write_zip(stage: &Path, dest: &Path) -> Result<()> {
    let mut zip = zip::ZipWriter::new(fs::File::create(dest)?);
    for path in files_in(stage)? {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(name, options)?;
        zip.write_all(&fs::read(&path)?)?;
    }
    zip.finish()?;
    Ok(())
}
