// src/archive.rs
use flate2::read::GzDecoder;
use std::io::Read;
use std::path::Path;
use tar::Archive;
use xz2::read::XzDecoder;

#[derive(Debug)]
pub enum ArchiveType {
    Gz,
    Xz,
    Unknown,
}

/*
    A function to determine an archive type by
     reading the magic bytes
    parameters:
        - bytes: part of the first bytes
*/
pub fn detect_type(bytes: &[u8]) -> ArchiveType {
    match bytes {
        [0x1F, 0x8B, ..] => ArchiveType::Gz,
        [0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00, ..] => ArchiveType::Xz,
        _ => ArchiveType::Unknown,
    }
}

/*
    A function to extract an archive
    parameters:
        - reader: the archive reader
        - archive_type: the archive type
        - output_dir: the output directory
*/
pub fn extract(
    reader: impl Read,
    archive_type: ArchiveType,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match archive_type {
        ArchiveType::Gz => {
            let decoder = GzDecoder::new(reader);
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        ArchiveType::Xz => {
            let decoder = XzDecoder::new(reader);
            let mut archive = Archive::new(decoder);
            archive.unpack(output_dir)?;
        }
        ArchiveType::Unknown => {
            return Err("unknown archive type".into());
        }
    }
    Ok(())
}
