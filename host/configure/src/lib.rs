use std::{fs, io, path::PathBuf};

use clap::Parser;
use serde::de::DeserializeOwned;
use snafu::Snafu;

use configurable::{ConfigStruct, configure::configure};

#[derive(Debug, Parser)]
pub struct Cli {
    /// Input binary to configure
    input: PathBuf,
    /// Configuration, in toml format
    config: PathBuf,
    /// Output binary
    output: PathBuf,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    Io { source: io::Error },
    #[snafu(transparent)]
    Toml { source: toml::de::Error },
    #[snafu(transparent)]
    Configure {
        source: configurable::configure::Error,
    },
}

pub fn main<T: ConfigStruct + DeserializeOwned>() -> Result<(), Error> {
    let Cli {
        input,
        config,
        output,
    } = Cli::parse();

    let mut input = fs::read(input)?;
    let config: T = toml::from_slice(&fs::read(config)?)?;

    configure(&mut input, &config)?;

    fs::write(output, input)?;

    Ok(())
}
