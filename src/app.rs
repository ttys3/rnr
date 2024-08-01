use clap::{Arg, ArgAction};
use clap::Command;
use std::ffi::{OsStr, OsString};

/// From file subcommand name.
pub const FROM_FILE_SUBCOMMAND: &str = "from-file";

/// To ASCII subcommand name.
pub const TO_ASCII_SUBCOMMAND: &str = "to-ascii";

/// Create application using clap. It sets all options and command-line help.
pub fn create_app<'a>() -> Command {
    // These commons args are shared by all commands.
    let common_args = [
        Arg::new("dry-run")
            .long("dry-run")
            .short('n')
            .action(ArgAction::SetTrue)
            .help("Only show what would be done (default mode)")
            .conflicts_with("force"),
        Arg::new("force")
            .long("force")
            .short('f')
            .action(ArgAction::SetTrue)
            .help("Make actual changes to files")
            .conflicts_with("dry-run"),
        Arg::new("backup")
            .long("backup")
            .short('b')
            .help("Generate file backups before renaming"),
        Arg::new("silent")
            .long("silent")
            .short('s')
            .help("Do not print any information"),
        Arg::new("color")
            .long("color")
            .value_parser(["always", "auto", "never"])
            .default_value("auto")
            .help("Set color output mode"),
        Arg::new("dump")
            .long("dump")
            .action(ArgAction::SetTrue)
            .help("Force dumping operations into a file even in dry-run mode")
            .conflicts_with("no-dump"),
        Arg::new("no-dump")
            .long("no-dump")
            .action(ArgAction::SetTrue)
            .help("Do not dump operations into a file")
            .conflicts_with("dump"),
    ];

    // Path related arguments.
    let path_args = [
        Arg::new("PATH(S)")
            .help("Target paths")
            .num_args(1..)
            .value_parser(clap::builder::StringValueParser::new())
            .required(true).index(3),
        Arg::new("include-dirs")
            .long("include-dirs")
            .short('D')
            .group("TEST")
            .help("Rename matching directories"),
        Arg::new("recursive")
            .long("recursive")
            .short('r')
            .action(ArgAction::SetTrue)
            .help("Recursive mode"),
        Arg::new("max-depth")
            .requires("recursive")
            .long("max-depth")
            .short('d')
            .num_args(1)
            .value_name("LEVEL")
            .value_parser(clap::builder::RangedI64ValueParser::<i64>::new())
            .help("Set max depth in recursive mode"),
        Arg::new("hidden")
            .requires("recursive")
            .long("hidden")
            .short('x')
            .help("Include hidden files and directories"),
    ];

    Command::new("rnr")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("EXPRESSION")
                .help("Expression to match (can be a regex)")
                .required(true)
                .value_parser(clap::builder::StringValueParser::new())
                .index(1),
        )
        .arg(
            Arg::new("REPLACEMENT")
                .help("Expression replacement (use single quotes for capture groups)")
                .required(true)
                .value_parser(clap::builder::StringValueParser::new())
                .index(2),
        )
        .arg(
            Arg::new("replace-limit")
                .long("replace-limit")
                .short('l')
                .num_args(1)
                .value_name("LIMIT")
                .default_value("1")
                .value_parser(clap::builder::RangedI64ValueParser::<usize>::new())
                .help("Limit of replacements, all matches if set to 0"),
        )
        .args(&common_args)
        .args(&path_args)
        .subcommand(
            Command::new(FROM_FILE_SUBCOMMAND)
                .args(&common_args)
                .arg(
                    Arg::new("DUMPFILE")
                        .num_args(1)
                        .required(true)
                        .value_name("DUMPFILE")
                        .value_parser(clap::builder::StringValueParser::new())
                        .index(1),
                )
                .arg(
                    Arg::new("undo")
                        .long("undo")
                        .short('u')
                        .help("Undo the operations from the dump file"),
                )
                .about("Read operations from a dump file"),
        )
        .subcommand(
            Command::new(TO_ASCII_SUBCOMMAND)
                .args(&common_args)
                .args(&path_args)
                .about("Replace file name UTF-8 chars with ASCII chars representation."),
        )
}

/// Check if the input provided is valid unsigned integer
fn is_integer(arg_value: String) -> Result<(), String> {
    match arg_value.parse::<usize>() {
        Ok(_) => Ok(()),
        Err(_) => Err("Value provided is not an integer".to_string()),
    }
}
