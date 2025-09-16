//! Tiny Rust minecraft server
#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::pedantic,
    clippy::all,
    clippy::ignore_without_reason
)]
#![allow(clippy::cast_possible_truncation)]

/// A writer.
pub trait McWriter {
    /// Write `bytes` to this stream. Should write all provided
    /// bytes or return an error.
    /// 
    /// # Errors
    /// If there's an error writing all bytes, return an error.
    fn write(&mut self, bytes: &[u8]) -> Result<(), anyhow::Error>;
}

/// A reader.
pub trait McReader {
    /// Read `bytes.len()` to this buffer. If there's an error reading
    /// that many bytes, return an error.
    /// 
    /// # Errors
    /// If there's an error reading that many bytes, return an error.
    /// The contents of the buffer is unspecified if an error is returned.
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), anyhow::Error>;
    /// Read a single byte.
    /// 
    /// # Errors
    /// Propogates errors from [`read`](McReader::read).
    fn read_byte(&mut self) -> Result<u8, anyhow::Error> {
        let mut out = [0u8];
        self.read(&mut out)?;
        Ok(out[0])
    }
}

/// A single serializable protocol item.
pub trait McProto<T = Self>: Sized {
    /// The metadata for reading.
    type Meta: Default;
    /// Write this item to the provided writer.
    /// 
    /// # Errors
    /// If the writer returns an error, propogate it.
    fn write(value: T, writer: &mut dyn McWriter) -> Result<(), anyhow::Error>;
    /// Read bytes from the reader with the provided metadata.
    /// 
    /// # Errors
    /// If the reader or deserializating encounters an error, propogate it.
    fn read(reader: &mut dyn McReader, meta: Self::Meta) -> Result<T, anyhow::Error>;
}

/// A single serializable protocol item.
pub trait McProtoSelf: Sized {
    /// The metadata for reading.
    type Meta: Default;
    /// Write this item to the provided writer.
    /// 
    /// # Errors
    /// If the writer returns an error, propogate it.
    fn write(self, writer: &mut dyn McWriter) -> Result<(), anyhow::Error>;
    /// Read bytes from the reader with the provided metadata.
    /// 
    /// # Errors
    /// If the reader or deserializating encounters an error, propogate it.
    fn read(reader: &mut dyn McReader, meta: Self::Meta) -> Result<Self, anyhow::Error>;
}

impl<T: McProtoSelf> McProto for T {
    type Meta = <Self as McProtoSelf>::Meta;
    
    fn write(value: Self, writer: &mut dyn McWriter) -> Result<(), anyhow::Error> {
        value.write(writer)
    }
    
    fn read(reader: &mut dyn McReader, meta: Self::Meta) -> Result<Self, anyhow::Error> {
        Self::read(reader, meta)
    }
    
}

pub mod types;