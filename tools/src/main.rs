use anyhow::Result;
use clap::Parser;

mod dump;
mod flags;
mod info;
mod init_database;

fn main() -> Result<()> {
    let cli = flags::ChewingCli::parse();
    match cli.subcommand {
        flags::ChewingCliCmd::InitDatabase(args) => init_database::run(args)?,
        flags::ChewingCliCmd::Info(args) => info::run(args)?,
        flags::ChewingCliCmd::Dump(args) => dump::run(args)?,
    }
    Ok(())
}
