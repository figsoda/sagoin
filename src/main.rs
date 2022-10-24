#![forbid(unsafe_code)]

use eyre::{Result, WrapErr};
use ignore::WalkBuilder;
use is_executable::IsExecutable;
use zip::{write::FileOptions, ZipWriter};

use std::{
    env::set_current_dir,
    fs::File,
    io::{self, Cursor},
};

use sagoin::{config::load_config, get_course_url};

fn main() -> Result<()> {
    let (cfg, mut state) = load_config()?;

    if let Some(dir) = &cfg.dir {
        set_current_dir(dir).wrap_err("failed to set current dir")?;
    }

    let props = java_properties::read(File::open(".submit").wrap_err("failed to read .submit")?)
        .wrap_err("failed to parse .submit")?;

    if !cfg.no_submit {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        zip.set_comment("");
        let regular = FileOptions::default();
        let executable = regular.unix_permissions(0o755);

        for entry in WalkBuilder::new(".").hidden(false).build() {
            let path = entry.wrap_err("failed to read entry")?.into_path();
            let path = path.strip_prefix(".")?;
            if !path.is_file() || matches!(path.file_name(), Some(name) if name == ".submitUser") {
                continue;
            }

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
        }

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
