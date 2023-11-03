use std::path::PathBuf;

use clap::{command, value_parser, Arg, ArgAction, Command};

/// Get the CLI definition as a [`clap::Command`].
pub fn get_cli() -> Command {
    let command = command!("docfmt")
        .arg(
            Arg::new("template")
                .value_parser(value_parser!(PathBuf))
                .required_unless_present("config")
                .help(concat!(
                    "Path to the main file defining the document structure. ",
                    "May be omitted if a config file is given."
                )),
        )
        .arg(
            Arg::new("output")
                .value_parser(value_parser!(PathBuf))
                .required_unless_present("config")
                .help(concat!(
                    "Path to the output file. ",
                    "May be omitted if a config file is given."
                )),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Append)
                .help("Path to a TOML file containing the configuration."),
        )
        .arg(
            Arg::new("include")
                .short('i')
                .long("include")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Append)
                .help(concat!(
                    "Path or file to include in the document. ",
                    "Can be used multiple times. ",
                    "Directories are traversed recursively. ",
                    "Files and directories are stripped from the path and the file extension."
                )),
        )
        .arg(
            Arg::new("extension")
                .short('e')
                .long("ext")
                .value_parser(value_parser!(String))
                .action(ArgAction::Append)
                .default_values(vec!["md", "markdown"])
                .value_delimiter(',')
                .help("Comma-separated list of file extensions to include in directories."),
        )
        .arg(
            Arg::new("data")
                .short('d')
                .long("data")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Append)
                .help(concat!(
                    "File containing data to be used in the document. ",
                    "May be a JSON or TOML file. The type is determined by the file extension. ",
                    "If defined multiple times, the data is merged.",
                )),
        )
        .arg(
            Arg::new("force")
                .short('f')
                .long("force")
                .action(ArgAction::SetTrue)
                .help("Force overwriting of the output file."),
        )
        .arg(
            Arg::new("strict")
                .short('s')
                .long("strict")
                .action(ArgAction::SetTrue)
                .help("Restrict accessing non-existing fields or indices in templates."),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Print verbose output."),
        );

    #[cfg(unix)]
    let command = command.arg(
        Arg::new("follow")
            .long("follow")
            .action(ArgAction::SetTrue)
            .help("Follow symlinks when traversing directories."),
    );

    command
}
