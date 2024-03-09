use anyhow::Result;

mod flags;
mod init_database;

fn main() -> Result<()> {
    let cli = flags::ChewingCli::from_env_or_exit();
    match cli.subcommand {
        flags::ChewingCliCmd::InitDatabase(args) => init_database::run(args)?,
    }
    Ok(())
}
