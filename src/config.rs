////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! The application configuration file.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// Local imports.
use crate::logger::LevelFilter;
use crate::logger::LoggerConfig;
use crate::logger::StdoutLogOutput;
use anyhow::Error;
use anyhow::Context;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

use log::*;

// Standard library imports.
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::borrow::Cow;
use std::collections::BTreeMap;


////////////////////////////////////////////////////////////////////////////////
// DEFAULT_CONFIG_PATH
////////////////////////////////////////////////////////////////////////////////
/// The default path to look for the [`Config`] file, relative to the app root.
///
/// [`Config`]: struct.Config.html
pub const DEFAULT_CONFIG_PATH: &'static str = ".atma-settings";

////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////
/// Application configuration data (tabuline file). Configures the logger and
/// defines files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The logger configuration.
    #[serde(default = "Config::default_logger_config")]
    pub logger_config: LoggerConfig,

    /// Module specific log levels.
    #[serde(default = "Config::default_log_levels")]
    pub log_levels: BTreeMap<Cow<'static, str>, LevelFilter>,
}


impl Config {
    /// Constructs a new `Config` with the default options.
    pub fn new() -> Self {
        Config::default()
    }

    /// Constructs a new `Config` with options read from the given file path.
    pub fn from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let file = File::open(path)
            .with_context(|| "Failed to open config file.")?;
        Config::from_file(file)
    }

    /// Constructs a new `Config` with options parsed from the given file.
    fn from_file(mut file: File) -> Result<Self, Error>  {
        Config::parse_ron_file(&mut file)
    }

    /// Parses a `Config` from a file using the RON format.
    fn parse_ron_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .with_context(|| "Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        let _ = file.read_to_end(&mut buf)
            .with_context(|| "Failed to read config file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .with_context(|| "Failed deserializing RON file")?;
        let config = Config::deserialize(&mut d)
            .with_context(|| "Failed parsing Ron file")?;
        d.end()
            .with_context(|| "Failed parsing Ron file")?;

        Ok(config) 
    }
    
    /// Normalizes paths in the config by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, base: &PathBuf) {
        match self.logger_config.log_path {
            Some(ref log_path) if log_path.is_relative() => {
                let log_path = base.clone().join(log_path);
                // Relative log file paths are relative to base.
                self.logger_config.log_path = Some(log_path);
            },
            _ => (),
        }
    }

    /// Returns the default [`LoggerConfig`].
    ///
    /// [`LoggerConfig`]: ../logger/struct.LoggerConfig.html
    #[inline(always)]
    fn default_logger_config() -> LoggerConfig {
        LoggerConfig {
            stdout_log_output: StdoutLogOutput::Colored,
            .. Default::default()
        }
    }

    /// Returns the default log levels for modules.
    #[inline(always)]
    fn default_log_levels() -> BTreeMap<Cow<'static, str>, LevelFilter> {
        Default::default()
    }

}

impl Default for Config {
    fn default() -> Self {
        Config {
            logger_config: Config::default_logger_config(),
            log_levels: Config::default_log_levels(),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\n\tlogger_config/stdout_log_output: {:?}",
            self.logger_config.stdout_log_output)?;
        writeln!(fmt, "\tlogger_config/level_filter: {:?}",
            self.logger_config.level_filter)
    }
}