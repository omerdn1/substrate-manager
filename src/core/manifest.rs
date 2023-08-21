use std::{
    fs::{self, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::PathBuf,
};

use anyhow::Ok;
use toml_edit::Document;

use crate::util::SubstrateResult;

#[derive(Debug)]
pub struct Manifest {
    path: PathBuf,
}

impl Manifest {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn read_document(&mut self) -> SubstrateResult<Document> {
        let toml = fs::read_to_string(&self.path)?;
        toml.parse()
            .map_err(|e| anyhow::Error::from(e).context("could not parse input as TOML"))
    }

    pub fn write_document(&mut self, document: Document) -> SubstrateResult<()> {
        let toml = document.to_string();
        let bytes = toml.as_bytes();

        self.write(bytes)
    }

    // TODO: decouple from this class and make as reusable utility function
    fn write(&mut self, bytes: &[u8]) -> SubstrateResult<()> {
        let path = &self.path;

        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(false)
            .open(path)?;

        file.seek(SeekFrom::Start(0))?;
        file.write_all(bytes)?;
        file.set_len(bytes.len() as u64)?;
        Ok(())
    }
}
