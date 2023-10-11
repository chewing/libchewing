use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// xtask - running project specific tasks
pub struct Cli {
    #[argh(subcommand)]
    pub cmd: Cmds,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Cmds {
    Build(CmdBuild),
    Test(CmdTest),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "build")]
/// Build project
pub struct CmdBuild {
    /// build with rust support
    #[argh(option, default = "true")]
    pub with_rust: bool,

    /// using hash implementation for user-dictionary, not compatible with --with-rust
    #[argh(option, default = "false")]
    pub with_hash: bool,

    /// show more information during build
    #[argh(option, default = "false")]
    pub verbose: bool,

    /// CMake build type (Release, Debug, RelWithDebInfo, etc.)
    #[argh(option, default = "String::from(\"Debug\")")]
    pub build_type: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "test")]
/// Running tests
pub struct CmdTest {
    /// CMake build type (Release, Debug, RelWithDebInfo, etc.)
    #[argh(option, default = "String::from(\"Debug\")")]
    pub build_type: String,

    /// execute cargo test
    #[argh(option, default = "true")]
    pub with_rust: bool,
}
