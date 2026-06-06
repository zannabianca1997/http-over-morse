extern crate std;

use elf::{ElfBytes, endian::AnyEndian};
use snafu::{OptionExt, Snafu};

use crate::ConfigStruct;

pub fn configure<T: ConfigStruct>(elf: &mut [u8], config: &T) -> Result<(), Error> {
    // Parse the elf, find the section header and run basic checks
    let section = ElfBytes::<AnyEndian>::minimal_parse(elf)?
        .section_header_by_name(T::SECTION)?
        .context(MissingSectionSnafu { name: T::SECTION })?;
    if section.sh_size != T::SIZE {
        return Err(Error::WrongSizeSection {
            expected: T::SIZE,
            found: section.sh_size,
        });
    }
    // TODO: check alignment? Endianness? Etc..

    // Range to modify
    let config_bytes_range =
        section.sh_offset as usize..(section.sh_offset + section.sh_size) as usize;

    // Copy the new config in the section
    elf[config_bytes_range].copy_from_slice(config.as_bytes());

    Ok(())
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("The elf file is invalid"), context(false))]
    InvalidElf { source: elf::ParseError },
    #[snafu(display("The config section {name} is missing"))]
    MissingSection { name: &'static str },
    #[snafu(display("The config section has a bad size"))]
    WrongSizeSection { expected: u64, found: u64 },
}
