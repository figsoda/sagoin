#![forbid(unsafe_code)]

use std::{
    env::{current_dir, set_current_dir},
    fs::File,
    io::{self, stdout, Cursor, Write},
    path::Path,
};

use eyre::{bail, Result, WrapErr};
use ignore::WalkBuilder;
use is_executable::IsExecutable;
use sagoin::{
    config::load_config,
    course::{get_course_url, print_course_info},
    warn,
};
use zip::{write::SimpleFileOptions, ZipWriter};

fn main() -> Result<()> {
    let (cfg, mut state) = load_config()?;

    if let Some(dir) = &cfg.dir {
        set_current_dir(dir).wrap_err("failed to set current dir")?;
    }

    let path = loop {
        let path = Path::new(".submit");
        if path.is_file() {
            break path;
        }

        let dir = current_dir().wrap_err("failed to get current directory")?;
        let Some(parent) = dir.parent() else {
            bail!("failed to find .submit");
        };

        warn!(
            state,
            "no .submit file found in {}, trying {}",
            dir.display(),
            parent.display(),
        );
        set_current_dir(parent).wrap_err("failed to set current directory")?;
    };

    if cfg.list_files {
        let mut out = stdout().lock();
        return walk(|path| writeln!(out, "{}", path.display()).map_err(Into::into));
    }

    let props = java_properties::read(File::open(path).wrap_err("failed to read .submit")?)
        .wrap_err("failed to parse .submit")?;

    if cfg.info {
        return print_course_info(&props, cfg.time_format);
    }

    if !cfg.no_submit {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        zip.set_comment("");
        let regular = SimpleFileOptions::default();
        let executable = regular.unix_permissions(0o755);

        walk(|path| {
            zip.start_file(
                path.to_string_lossy(),
                if path.is_executable() {
                    executable
                } else {
                    regular
                },
            )
            .wrap_err("failed to write to the zip file")?;

            io::copy(
                &mut File::open(path)
                    .wrap_err_with(|| format!("failed to read {}", path.display()))?,
                &mut zip,
            )
            .wrap_err("failed to write to the zip file")?;

            Ok(())
        })?;

        state.submit(
            File::open(".submitUser")
                .ok()
                .and_then(|file| java_properties::read(file).ok())
                .unwrap_or_default(),
            &props,
            &cfg,
            &zip.finish()
                .wrap_err("failed to finish writing to the zip file")?
                .into_inner(),
        )?;
    }

    if cfg.open {
        webbrowser::open(&get_course_url(&props)?).wrap_err("failed to open the web browser")?;
    }

    Ok(())
}

fn walk(mut f: impl FnMut(&Path) -> Result<()>) -> Result<()> {
    for entry in WalkBuilder::new(".")
        .hidden(false)
        .filter_entry(|entry| entry.depth() != 1 || entry.file_name() != ".git")
        .build()
    {
        let path = entry.wrap_err("failed to read entry")?.into_path();
        let path = path.strip_prefix(".")?;
        if path.is_file() && matches!(path.file_name(), Some(name) if name != ".submitUser") {
            f(path)?;
        }
    }

    Ok(())
}
