use std::{
    fs::File,
    io::read_to_string,
    os::unix::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use clap::ArgMatches;
use handlebars::{Handlebars, TemplateError};
use log::{info, warn, error};
use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Debug, Default, Deserialize)]
#[serde(rename = "Config")]
struct ConfigRead {
    template: Option<PathBuf>,
    output: Option<PathBuf>,
    #[serde(default)]
    force: bool,
    #[serde(default)]
    follow: bool,
    #[serde(default)]
    strict: bool,
    #[serde(default)]
    verbose: bool,
    #[serde(default)]
    include: Vec<PathBuf>,
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    datafiles: Vec<PathBuf>,
    data: Option<toml::Value>,
}

#[derive(Debug, Default)]
pub struct Config {
    template: PathBuf,
    output: PathBuf,
    force: bool,
    follow: bool,
    strict: bool,
    verbose: bool,
    include: Vec<PathBuf>,
    extensions: Vec<String>,
    datafiles: Vec<PathBuf>,
    data: serde_json::Value,
}

impl Config {
    #[inline]
    pub fn log_level(&self) -> log::Level {
        if self.verbose {
            #[cfg(debug_assertions)]
            return log::Level::Debug;
            #[cfg(not(debug_assertions))]
            return log::Level::Info;
        }
        log::Level::Warn
    }

    #[allow(clippy::result_large_err)]
    pub fn new_registry(&self) -> Option<Handlebars> {
        let mut failed = false;
        let mut registry = Handlebars::new();

        if self.strict {
            registry.set_strict_mode(true);
            info!("Enabled strict mode");
        }
        if self.follow {
            info!("Enabled follow mode");
        }
        if let Err(err) = registry.register_template_file("main", &self.template) {
            error!("Unable to register main template: {:?}", self.template);
            error!("{}", err);
            failed = true;
        }
        info!("Registered main template: {:?}", self.template);
        for path in &self.include {
            let path = path.to_owned();

            if path.is_dir() {
                info!("Walking directory: {:?}", path);
                info!("Including files with extensions: {:?}", self.extensions);
                let root = path.parent().unwrap_or(Path::new("")).to_owned();
                for entry in WalkDir::new(path).follow_links(self.follow) {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => {
                            error!("Unable to read file: {:?}", err.path());
                            error!("{}", err);
                            failed = true;
                            continue;
                        }
                    };

                    if let Some(ext) = entry.path().extension() {
                        let ext = match ext.to_str() {
                            Some(ext) => ext.to_owned(),
                            None => {
                                error!("Unable to read extension of file: {:?}", entry.path());
                                warn!("File extension is not valid UTF-8");
                                failed = true;
                                continue;
                            }
                        };

                        if !self.extensions.contains(&ext) {
                            continue;
                        }
                    } else {
                        continue;
                    }

                    let meta = match entry.metadata() {
                        Ok(meta) => meta,
                        Err(err) => {
                            error!("Unable to read metadata of file: {:?}", err.path());
                            error!("{}", err);
                            failed = true;
                            continue;
                        }
                    };
                    if meta.is_file() {
                        info!("Reading file: {:?}", entry.path());
                        let name = entry.path();

                        if let Some(stem) = name.file_stem() {
                            if stem.as_bytes().first() == Some(&b'.') {
                                continue;
                            }
                        } else {
                            error!("Unable to register file: {:?}", entry.path());
                            warn!("File name is not valid UTF-8");
                            failed = true;
                            continue;
                        }

                        let name = name.strip_prefix(&root).unwrap();
                        let name = name.with_extension("");
                        #[cfg(windows)]
                        let name = name.to_str().unwrap().replace('\\', "/");
                        #[cfg(unix)]
                        let name = match name.to_str() {
                            Some(name) => name,
                            None => {
                                error!("Unable to register file: {:?}", entry.path());
                                warn!("File path is not valid UTF-8");
                                failed = true;
                                continue;
                            }
                        };
                        if let Err(err) = registry.register_template_file(name.as_ref(), entry.path()) {
                            error!("Unable to register file: {:?}", entry.path());
                            error!("{}", err);
                            failed = true;
                            continue;
                        }
                        info!("Registered template: {:?}", name);
                    }
                }
            } else if path.is_file() {
                info!("Reading file: {:?}", &path);
                let name = path.with_extension("");
                let name = name.file_name().unwrap();
                #[cfg(windows)]
                let name = name.to_str().unwrap().replace('\\', "/");
                #[cfg(unix)]
                let name = name.to_str().unwrap();
                if let Err(err) =  registry.register_template_file(name.as_ref(), &path) {
                    error!("Unable to register file: {:?}", path);
                    error!("{}", err);
                    failed = true;
                    continue;
                }
                info!("Registered template: {:?}", name);
            }
        }
        if failed {
            return None;
        }
        Some(registry)
    }

    #[allow(clippy::result_large_err)]
    pub fn read_data(&self) -> Option<serde_json::Value> {
        let mut failed = false;
        let mut data = self.data.clone();

        macro_rules! log_error {
            ($path:expr, $err:expr) => {{
                error!("Unable to read data file: {:?}", $path);
                error!("{}", $err);
                failed = true;
                continue;
            }};
        }

        for path in &self.datafiles {
            info!("Reading data file: {:?}", path);
            let file = match File::open(path) {
                Ok(file) => file,
                Err(err) => {
                    error!("Unable to open data file: {:?}", path);
                    error!("{}", err);
                    failed = true;
                    continue;
                }
            };
            let content = match read_to_string(file) {
                Ok(content) => content,
                Err(err) => log_error!(path, err),
            };
            let value = if path.extension() == Some("json".as_ref()) {
                match serde_json::from_str(&content) {
                    Ok(value) => value,
                    Err(err) => log_error!(path, err),
                }
            } else if path.extension() == Some("toml".as_ref()) {
                let value = match toml::from_str::<toml::Value>(&content) {
                    Ok(value) => value,
                    Err(err) => log_error!(path, err),
                };

                match serde_json::to_value(value) {
                    Ok(value) => value,
                    Err(err) => log_error!(path, err),
                }
            } else {
                error!("Unable to read data file: {:?}", path);
                error!("Unsupported file extension");
                failed = true;
                continue;
            };

            Self::merge(&mut data, value);
        }

        if failed {
            return None;
        }
        Some(data)
    }

    #[allow(clippy::result_large_err)]
    pub fn write_output(&self, content: String) -> bool {
        info!("Writing output file: {:?}", self.output);
        if self.output.exists() && !self.force {
            error!("Output file already exists: {:?}", self.output);
            return false;
        }

        if let Err(err) = std::fs::write(&self.output, content) {
            error!("Unable to write output file: {:?}", self.output);
            error!("{}", err);
            return false;
        }
        true
    }

    fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
        // CREDITS: https://stackoverflow.com/a/54118457
        if let serde_json::Value::Object(a) = a {
            if let serde_json::Value::Object(b) = b {
                for (k, v) in b {
                    if v.is_null() {
                        a.remove(&k);
                    } else {
                        Self::merge(a.entry(k).or_insert(serde_json::Value::Null), v);
                    }
                }
                return;
            }
        }

        *a = b;
    }
}

impl TryFrom<ArgMatches> for Config {
    type Error = ConfigError;

    fn try_from(matches: ArgMatches) -> Result<Self, Self::Error> {
        let mut config = match matches.get_one::<PathBuf>("config") {
            Some(path) => {
                let content =
                    read_to_string(File::open(path).map_err(ConfigError::ConfigFileReadError)?)
                        .map_err(ConfigError::ConfigFileReadError)?;
                toml::from_str::<ConfigRead>(&content).map_err(ConfigError::InvalidConfig)?
            }
            None => ConfigRead::default(),
        };

        config.template = matches
            .get_one::<PathBuf>("template")
            .cloned()
            .or(config.template);
        config.output = matches
            .get_one::<PathBuf>("output")
            .cloned()
            .or(config.output);
        config.force = if matches.get_flag("force") {
            true
        } else {
            config.force
        };
        #[cfg(unix)]
        {
            config.follow = if matches.get_flag("follow") {
                true
            } else {
                config.follow
            };
        }
        #[cfg(not(unix))]
        {
            config.follow = false;
        }
        config.strict = if matches.get_flag("strict") {
            true
        } else {
            config.strict
        };
        config.verbose = if matches.get_flag("verbose") {
            true
        } else {
            config.verbose
        };
        config.include.extend(
            matches
                .get_many::<PathBuf>("include")
                .unwrap_or_default()
                .map(PathBuf::from),
        );
        config.extensions.extend(
            matches
                .get_many::<String>("extension")
                .unwrap_or_default()
                .map(String::from),
        );
        config.datafiles.extend(
            matches
                .get_many::<PathBuf>("data")
                .unwrap_or_default()
                .map(PathBuf::from),
        );

        Self::try_from(config)
    }
}

impl TryFrom<ConfigRead> for Config {
    type Error = ConfigError;

    fn try_from(config: ConfigRead) -> Result<Self, Self::Error> {
        Ok(Config {
            template: config.template.ok_or(ConfigError::MissingTemplate)?,
            output: config.output.ok_or(ConfigError::MissingOutput)?,
            force: config.force,
            follow: config.follow,
            strict: config.strict,
            verbose: config.verbose,
            include: config.include,
            extensions: config.extensions,
            datafiles: config.datafiles,
            data: config
                .data
                .map_or(serde_json::Value::Object(serde_json::Map::default()), |v| {
                    serde_json::to_value(v).unwrap()
                }),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unable to read config file: {0}")]
    ConfigFileReadError(std::io::Error),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(toml::de::Error),
    #[error("Missing template file")]
    MissingTemplate,
    #[error("Missing output file")]
    MissingOutput,
    #[error("Unable to read template: {0}")]
    TemplateError(#[from] TemplateError),
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn create_registry() {
        let config = Config {
            template: PathBuf::from("tests/templates/main.hbs"),
            output: PathBuf::from("tests/output/main.md"),
            force: false,
            follow: false,
            strict: false,
            verbose: false,
            include: vec![
                PathBuf::from("tests/templates/input1"),
                PathBuf::from("tests/templates/input2"),
                PathBuf::from("tests/templates/file.hbs"),
            ],
            extensions: vec!["hbs".into(), "md".into()],
            datafiles: vec![],
            data: serde_json::Value::Object(serde_json::Map::default()),
        };

        let registry = config.new_registry();
        assert!(registry.is_some());
        let registry = registry.unwrap();

        assert!(registry.get_template("main").is_some());
        assert!(registry.get_template("input1/file").is_some());
        assert!(registry.get_template("input1/.hidden").is_none());
        assert!(registry.get_template("input1/subdir/file").is_some());
        assert!(registry.get_template("input2/file").is_some());
        assert!(registry.get_template("input2/.hidden").is_none());
        assert!(registry.get_template("file").is_some());

        let data = config.read_data();
        assert!(data.is_some());
        let data = data.unwrap();

        let content = registry.render("main", &data);
        assert!(content.is_ok());
        let content = content.unwrap();

        assert_eq!(content, "Hello World!\nGoodbye!\nFor now!");
    }

    #[test]
    fn read_data() {
        let config = Config {
            template: PathBuf::from("tests/templates/main.hbs"),
            output: PathBuf::from("tests/output/main.md"),
            force: false,
            follow: false,
            strict: false,
            verbose: false,
            include: vec![],
            extensions: vec![],
            datafiles: vec![
                PathBuf::from("tests/data/data1.toml"),
                PathBuf::from("tests/data/data2.json"),
            ],
            data: serde_json::Value::Object(serde_json::Map::default()),
        };

        let data = config.read_data();
        assert!(data.is_some());
        let data = data.unwrap();
        let expected = json!({
            "cities": [
              "colombo"
            ],
            "person": {
              "firstName": "Jane"
            },
            "title": "This is another title"
        });

        assert_eq!(data, expected);
    }

    #[test]
    fn write_output() {
        let config = Config {
            template: PathBuf::from("tests/templates/main.hbs"),
            output: PathBuf::from("tests/output/main.md"),
            force: true,
            follow: false,
            strict: false,
            verbose: false,
            include: vec![],
            extensions: vec![],
            datafiles: vec![],
            data: serde_json::Value::Object(serde_json::Map::default()),
        };

        let content = "Hello World!\nGoodbye!\nFor now!".to_owned();
        let success = config.write_output(content);
        assert!(success);
        let content = std::fs::read_to_string("tests/output/main.md");
        assert!(content.is_ok());
        let content = content.unwrap();
        assert_eq!(content, "Hello World!\nGoodbye!\nFor now!");

        let config = Config {
            template: PathBuf::from("tests/templates/main.hbs"),
            output: PathBuf::from("tests/output/main.md"),
            force: false,
            follow: false,
            strict: false,
            verbose: false,
            include: vec![],
            extensions: vec![],
            datafiles: vec![],
            data: serde_json::Value::Object(serde_json::Map::default()),
        };

        let content = "Hello World!\nGoodbye!\nFor now!".to_owned();
        let success = config.write_output(content);
        assert!(!success);

    }
}
