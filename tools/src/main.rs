use anyhow::Result;
use clap::Parser;

mod dump;
mod flags;
mod info;
mod init_database;

fn main() -> Result<()> {
    #[cfg(feature = "mangen")]
    {
        use clap::CommandFactory;
        if let Ok(_) = std::env::var("UPDATE_MANPAGE") {
            clap_mangen::generate_to(
                flags::ChewingCli::command(),
                std::env::args().nth(1).unwrap(),
            )?;
            return Ok(());
        }
    }
    let cli = flags::ChewingCli::parse();
    match cli.subcommand {
        flags::ChewingCliCmd::InitDatabase(args) => init_database::run(args)?,
        flags::ChewingCliCmd::Info(args) => info::run(args)?,
        flags::ChewingCliCmd::Dump(args) => dump::run(args)?,
    }
    Ok(())
}
