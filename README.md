# Document Formatter

![Build Status](https://img.shields.io/github/actions/workflow/status/typedduck/docfmt/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/docfmt)](https://crates.io/crates/docfmt)
[![Crates.io](https://img.shields.io/crates/d/docfmt)](https://crates.io/crates/docfmt)

This is a tool to format documents using a template and data. Multiple data files can be merged. Paths and files can be included and referenced in the template.

The data and input files are passed to the template engine [Handlebars](https://handlebarsjs.com/).

## Template

The template file may be in any format supported by [Handlebars](https://handlebarsjs.com/). The implementation used is [handlebars-rust](https://docs.rs/handlebars/latest/handlebars/). The feature `rust-embed` is not enabled.

Handlebars is a versatile template engine. It supports a wide range of features. The documentation for Handlebars is extensive and can be found [here](https://handlebarsjs.com/guide/).

## Usage

```bash
docfmt [OPTIONS] <TEMPLATE> <OUTPUT>
```

## Options

### `-c`, `--config`

Path to a TOML file containing the configuration. The configuration file can be used to define the template, output, data, and includes. The command line arguments take precedence over the configuration file.

### `-i`, `--include`

Path or file to include in the document. Can be used multiple times. Directories are traversed recursively. Files and directories are stripped from the path and the file extension. Dotfiles are ignored when traversing directories. The files are included in the order they are defined.

On Windows, the stripped path naming the template are converted to use forward slashes as well.

### `-e`, `--ext`

Comma-separated list of file extensions to include in directories. Defaults to `md,markdown`.

### `-d`, `--data`

Path or file to include in the document. Can be used multiple times. Directories are traversed recursively. Data may be defined in JSON or TOML format. The type is determined by the file extension. If defined multiple times, the data is merged. Merging is done in the sequence the files are defined. The last file takes precedence over the previous ones.

### `-f`, `--force`

Overwrite the output file if it already exists.

### `-v`, `--verbose`

Enable verbose logging. If defined the log level is set to `debug` if the program is compiled in debug mode, otherwise it is set to `info`. If not defined the log level is set to `warn`. The log is written to stderr.

### `-s`, `--strict`

Restrict accessing non-existing fields or indices in templates. If defined the program will exit with an error if a field or index is accessed that does not exist. If not defined the program will ignore such accesses.

### `--follow`

Follow symbolic links when traversing directories. This option is only available on Unix systems.

### `-V`, `--version`

Print version information.

### `-h`, `--help`

Print help information.

## Arguments

### `<TEMPLATE>`

Path to the template file. The template file may be in any format supported by [Handlebars](https://handlebarsjs.com/). The template file may be omitted if the template is defined in the configuration file.

### `<OUTPUT>`

Path to the output file. The output file may be omitted if the output is defined in the configuration file.

## Configuration

The configuration file is a TOML file. The following keys are supported:

```toml
template = "<path to template>"
output = "<path to output>"
force = false
follow = false
verbose = false
strict = false
include = ["<file to include>", "<path to include>"]
ext = ["md", "markdown"]
datafiles = ["<path to json-file>", "<path to toml-file>"]

[data]
title = "My title"
```

The `template` and `output` keys are required. The `force`, `follow`, `verbose`, and `strict` keys are optional and default to `false`. The `include` and `ext` keys are optional and default to `[]` and `["md", "markdown"]` respectively. The `datafiles` key is optional and defaults to `[]`. The `data` key is optional and defaults to `{}`.
