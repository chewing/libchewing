use anyhow::{anyhow, Context, Result};
use std::{
    env,
    path::{Path, PathBuf},
};
use xshell::{cmd, Shell};

mod cli;

use cli::{Cli, Cmds};

struct BuildOpts {
    with_rust: bool,
    with_hash: bool,
    with_coverage: bool,
    verbose: bool,
    build_type: String,
}

impl BuildOpts {
    fn run(&self, sh: &Shell) -> Result<()> {
        // Project not fully initiallized
        if !sh.path_exists("./cmake/corrosion") {
            cmd!(sh, "git submodule update --init --recursive")
                .run()
                .with_context(|| "cannot initialize cmake submodule")?
        }

        // Configure CMake
        let with_rust: String = format!("-DWITH_RUST={}", self.with_rust);
        let with_sqlite3: String = format!("-DWITH_SQLITE3={}", !self.with_hash);
        let with_coverage: String = format!("-DENABLE_GCOV={}", self.with_coverage);
        let build_type: String = format!("-DCMAKE_BUILD_TYPE={}", self.build_type);
        cmd!(
            sh,
            "cmake -B ./build {with_rust} {build_type} {with_sqlite3} {with_coverage}"
        )
        .run()
        .with_context(|| "cannot configure cmake")?;

        // Show build config
        if self.verbose {
            cmd!(sh, "cmake -B ./build -LA")
                .run()
                .with_context(|| "cannot show build config")?;
        }

        // Build Project
        let build_type = format!("--config={}", self.build_type);
        let verbose = if self.verbose { Some("-v") } else { None };
        cmd!(sh, "cmake --build ./build {build_type} {verbose...}")
            .run()
            .with_context(|| "build failed")?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let app: Cli = argh::from_env();

    let sh = Shell::new()?;
    sh.change_dir(project_root());

    match app.cmd {
        Cmds::Build(ref cmd) => {
            if cmd.with_hash && cmd.with_rust {
                return Err(anyhow!(
                    "--with_hash and --with_rust should not be used together"
                ));
            }
            BuildOpts {
                with_rust: cmd.with_rust,
                with_hash: cmd.with_hash,
                with_coverage: cmd.with_coverage,
                verbose: cmd.verbose,
                build_type: cmd.build_type.clone(),
            }
            .run(&sh)?;
            Ok(())
        }
        Cmds::Test(ref cmd) => {
            if !sh.path_exists("./build") {
                return Err(anyhow!("build project before running tests"));
            }

            if cmd.with_rust {
                cmd!(sh, "cargo test").run()?;
            }

            sh.change_dir("./build");
            let build_type = cmd.build_type.clone();
            cmd!(sh, "ctest -C {build_type} --output-on-failure").run()?;
            Ok(())
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}
