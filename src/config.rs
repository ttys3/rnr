use std::ffi::OsString;
use std::io::IsTerminal;
use std::ops::Deref;
use crate::app::{create_app, FROM_FILE_SUBCOMMAND, TO_ASCII_SUBCOMMAND};
use clap::ArgMatches;
use crate::output::Printer;
use regex::Regex;
use std::sync::Arc;
use clap::builder::TypedValueParser;

/// This module is defined Config struct to carry application configuration. This struct is created
/// from the parsed arguments from command-line input using `clap`. Only UTF-8 valid arguments are
/// considered.
pub struct Config {
    pub force: bool,
    pub backup: bool,
    pub dirs: bool,
    pub dump: bool,
    pub run_mode: RunMode,
    pub replace_mode: ReplaceMode,
    pub printer: Printer,
}

impl Config {
    pub fn new() -> Result<Arc<Config>, String> {
        let config = match parse_arguments() {
            Ok(config) => config,
            Err(err) => return Err(err),
        };
        Ok(Arc::new(config))
    }
}

pub enum RunMode {
    Simple(Vec<String>),
    Recursive {
        paths: Vec<String>,
        max_depth: Option<usize>,
        hidden: bool,
    },
    FromFile {
        path: String,
        undo: bool,
    },
}

pub enum ReplaceMode {
    RegExp {
        expression: Regex,
        replacement: String,
        limit: usize,
    },
    ToASCII,
}

/// Application commands
#[derive(Debug, PartialEq)]
pub enum AppCommand {
    Root,
    FromFile,
    ToASCII,
}

impl AppCommand {
    pub fn from_str(name: &str) -> Result<AppCommand, String> {
        match name {
            "" => Ok(AppCommand::Root),
            FROM_FILE_SUBCOMMAND => Ok(AppCommand::FromFile),
            TO_ASCII_SUBCOMMAND => Ok(AppCommand::ToASCII),
            _ => Err(format!("Non-registered subcommand '{}'", name)),
        }
    }
}

struct ArgumentParser<'a> {
    matches: &'a ArgMatches,
    printer: &'a Printer,
    command: &'a AppCommand,
}

impl ArgumentParser<'_> {
    fn parse_run_mode(&self) -> Result<RunMode, String> {
        if let AppCommand::FromFile = self.command {
            return Ok(RunMode::FromFile {
                path: String::from(self.matches.get_one::<String>("DUMPFILE").unwrap_or(&String::new())),
                undo: self.matches.contains_id("undo"),
            });
        }

        // Detect run mode and set parameters accordingly
        let input_paths: Vec<String> = self
            .matches
            .get_many::<String>("PATH(S)")
            .unwrap_or_default()
            .map(String::from)
            .collect();

        if self.matches.contains_id("recursive") {
            let max_depth = if self.matches.contains_id("max-depth") {
                Some(
                    *self.matches
                        .get_one::<usize>("max-depth")
                        .unwrap_or(&0),
                )
            } else {
                None
            };

            Ok(RunMode::Recursive {
                paths: input_paths,
                max_depth,
                hidden: self.matches.contains_id("hidden"),
            })
        } else {
            Ok(RunMode::Simple(input_paths))
        }
    }

    fn parse_replace_mode(&self) -> Result<ReplaceMode, String> {
        if let AppCommand::ToASCII = self.command {
            return Ok(ReplaceMode::ToASCII);
        }

        // Get and validate regex expression and replacement from arguments
        let expression = match Regex::new(self.matches.get_one::<String>("EXPRESSION").unwrap_or(&String::new()).deref()) {
            Ok(expr) => expr,
            Err(err) => {
                return Err(format!(
                    "{}Bad expression provided\n\n{}",
                    self.printer.colors.error.paint("Error: "),
                    self.printer.colors.error.paint(err.to_string())
                ));
            }
        };
        let replacement = String::from(self.matches.get_one::<String>("REPLACEMENT").unwrap_or(&String::new()).deref());

        let limit = *self
            .matches
            .get_one::<usize>("replace-limit")
            .unwrap_or(&0);

        Ok(ReplaceMode::RegExp {
            expression,
            replacement,
            limit,
        })
    }
}

/// Parse arguments and do some checking.
fn parse_arguments() -> Result<Config, String> {
    let app = create_app();
    let matches = app.get_matches();
    let (command, matches) = match matches.subcommand() {
        Some((name, submatches)) => (AppCommand::from_str(name)?, submatches),
        None  => (AppCommand::Root, &matches), // Always defaults to root if no submatches found.
        _ => {
            return Err("No command provided".to_string());
        }
    };

    // Set dump defaults: write in force mode and do not in dry-run unless it is explicitly asked
    let dump = if matches.contains_id("force") {
        !matches.contains_id("no-dump")
    } else {
        matches.contains_id("dump")
    };

    let printer = if matches.contains_id("silent") {
        Printer::silent()
    } else {
        match matches.get_one::<String>("color").unwrap_or(&"auto".to_string()).deref() {
            "always" => Printer::color(),
            "never" => Printer::no_color(),
            _ => detect_output_color(), // Ignore non-valid values and use auto.
        }
    };

    let argument_parser = ArgumentParser {
        printer: &printer,
        matches,
        command: &command,
    };

    let run_mode = argument_parser.parse_run_mode()?;
    let replace_mode = argument_parser.parse_replace_mode()?;

    Ok(Config {
        force: matches.contains_id("force"),
        backup: matches.contains_id("backup"),
        dirs: matches.contains_id("include-dirs"),
        dump,
        run_mode,
        replace_mode,
        printer,
    })
}

/// Detect if output must be colored and returns a properly configured printer.
fn detect_output_color() -> Printer {
    if std::io::stdout().is_terminal() {
        #[cfg(not(windows))]
        {
            Printer::color()
        }
        // Enable color support for Windows 10
        #[cfg(windows)]
        {
            use ansi_term;
            match ansi_term::enable_ansi_support() {
                Ok(_) => Printer::color(),
                Err(_) => Printer::no_color(),
            }
        }
    } else {
        Printer::no_color()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn app_command_from_str() {
        assert_eq!(AppCommand::from_str("").unwrap(), AppCommand::Root);
        assert_eq!(
            AppCommand::from_str(FROM_FILE_SUBCOMMAND).unwrap(),
            AppCommand::FromFile
        );
        assert_eq!(
            AppCommand::from_str(TO_ASCII_SUBCOMMAND).unwrap(),
            AppCommand::ToASCII
        );
    }

    #[test]
    #[should_panic]
    fn app_command_from_str_unknown_error() {
        AppCommand::from_str("this-command-does-not-exists").unwrap();
    }
}
