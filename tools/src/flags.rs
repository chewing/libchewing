use std::path::PathBuf;

use chewing::dictionary::DictionaryUsage;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about)]
pub(crate) struct ChewingCli {
    #[command(subcommand)]
    pub(crate) subcommand: ChewingCliCmd,
}

#[derive(Subcommand)]
pub(crate) enum ChewingCliCmd {
    /// Create a new dictionary file
    #[command(alias = "init")]
    InitDatabase(InitDatabase),
    /// Display information about the dictionary
    Info(Info),
    /// Dump the dictionary entries into tsi.src formatted stream
    Dump(Dump),
}

#[derive(Args)]
pub(crate) struct InitDatabase {
    /// Choose the underlying database implementation.
    #[arg(short('t'), long, value_enum, default_value = "trie")]
    pub(crate) db_type: DbType,
    /// Name of the phrase dictionary
    #[arg(short, long, default_value = "我的詞庫")]
    pub(crate) name: String,
    /// Copyright information of the dictionary
    #[arg(short, long, default_value = "Unknown")]
    pub(crate) copyright: String,
    /// License information of the dictionary
    #[arg(short, long, default_value = "Unknown")]
    pub(crate) license: String,
    /// Version of the dictionary
    #[arg(short('r'), long, default_value = "1.0.0")]
    pub(crate) version: String,
    /// The usage type of the dictionary
    #[arg(short, long, default_value = "unknown")]
    pub(crate) usage: DictionaryUsage,
    /// Skip invalid lines
    #[arg(short, long)]
    pub(crate) skip_invalid: bool,
    /// Attempt to fix simple spelling errors
    #[arg(long)]
    pub(crate) fix: bool,
    /// Read the dictionary source as CSV with header
    #[arg(long)]
    pub(crate) csv: bool,
    /// Path to the dictionary source file
    pub(crate) tsi_src: PathBuf,
    /// Path to the output file
    pub(crate) output: PathBuf,
}

#[derive(Args)]
pub(crate) struct Info {
    /// Location of the dictionary file
    #[arg(short, long, required_unless_present_any(["user", "system"]))]
    pub(crate) path: Option<PathBuf>,
    /// Display information of detected user dictionary
    #[arg(short, long)]
    pub(crate) user: bool,
    /// Display information of detected system dictionary
    #[arg(short, long)]
    pub(crate) system: bool,
    /// Output in JSON format
    #[arg(short, long)]
    pub(crate) json: bool,
}

#[derive(Args)]
pub(crate) struct Dump {
    /// Location of the dictionary file
    pub(crate) path: PathBuf,
    /// Location of the output file
    ///
    /// If OUTPUT equals to `-` then standard output will be used.
    pub(crate) output: Option<PathBuf>,
    /// Output CSV format
    #[arg(long)]
    pub(crate) csv: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum DbType {
    Trie,
    Sqlite,
}
