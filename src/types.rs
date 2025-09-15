//! Minecraft protocol data types.

use core::{
    num::NonZero,
    ops::{Deref, DerefMut},
};

use crate::{McProto, McProtoSelf, McReader, McWriter};

/// Macro for generating a `McProto` implemetation for number types.
macro_rules! int_impl {
    ($ty:ty, $bits:expr) => {
        impl McProtoSelf for $ty {
            type Meta = ();
            fn write(self, writer: &mut dyn McWriter) -> Result<(), &'static str> {
                writer.write(&self.to_be_bytes())
            }
            fn read(reader: &mut dyn McReader, (): ()) -> Result<Self, &'static str> {
                let mut bytes = [0x00u8; ($bits) as usize / 8];
                reader.read(&mut bytes)?;
                Ok(<$ty>::from_be_bytes(bytes))
            }
        }
    };
}

impl McProtoSelf for bool {
    type Meta = ();
    fn write(self, writer: &mut dyn McWriter) -> Result<(), &'static str> {
        writer.write(&[u8::from(!self)])
    }
    fn read(reader: &mut dyn McReader, (): ()) -> Result<Self, &'static str> {
        let mut bytes = [0xFFu8];
        reader.read(&mut bytes)?;
        Ok(match bytes[0] {
            0x00 => false,
            0x01 => true,
            _ => return Err("bad boolean value"),
        })
    }
}

impl McProtoSelf for u8 {
    type Meta = ();
    fn write(self, writer: &mut dyn McWriter) -> Result<(), &'static str> {
        writer.write(&[self])
    }
    fn read(reader: &mut dyn McReader, (): ()) -> Result<Self, &'static str> {
        let mut bytes = [0x00u8];
        reader.read(&mut bytes)?;
        Ok(bytes[0])
    }
}

int_impl!(i8, 8);

int_impl!(u16, 16);
int_impl!(i16, 16);

int_impl!(u32, 32);
int_impl!(i32, 32);

int_impl!(u64, 64);
int_impl!(i64, 64);

int_impl!(f32, 32);
int_impl!(f64, 64);

/// A non-zero length.
#[derive(Clone, Copy, PartialEq, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct Length(pub NonZero<usize>);

impl Default for Length {
    fn default() -> Self {
        Self(unsafe { NonZero::new_unchecked(1) })
    }
}

impl Deref for Length {
    type Target = NonZero<usize>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Length {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A varint or varlong.
#[derive(Clone, Copy, Debug, Default)]
pub struct VarNum;

impl VarNum {
    /// The bits used for data.
    pub const SEGMENT_BITS: u8 = 0x7F;
    /// The bit used as a marker for continuing.
    pub const CONTINUE_BIT: u8 = 0x80;
}

impl McProto<u32> for VarNum {
    fn write(mut value: u32, writer: &mut dyn McWriter) -> Result<(), &'static str> {
        loop {
            if (value & !u32::from(Self::SEGMENT_BITS)) == 0 {
                return value.write(writer);
            }
            ((value as u8 & Self::SEGMENT_BITS) | Self::CONTINUE_BIT).write(writer)?;

            value >>= 7;
        }
    }
    fn read(reader: &mut dyn McReader, (): Self::Meta) -> Result<u32, &'static str> {
        let mut value = 0u32;
        let mut position = 0u8;

        loop {
            let byte = reader.read_byte()?;
            value |= u32::from(byte & Self::SEGMENT_BITS) << position;
            if (byte & Self::CONTINUE_BIT) == 0 {
                return Ok(value);
            }

            position += 7;

            if position >= 32 {
                return Err("VarInt is too large");
            }
        }
    }
    type Meta = ();
}

impl McProto<u64> for VarNum {
    fn write(mut value: u64, writer: &mut dyn McWriter) -> Result<(), &'static str> {
        loop {
            if (value & !u64::from(Self::SEGMENT_BITS)) == 0 {
                return value.write(writer);
            }
            ((value as u8 & Self::SEGMENT_BITS) | Self::CONTINUE_BIT).write(writer)?;

            value >>= 7;
        }
    }
    fn read(reader: &mut dyn McReader, (): Self::Meta) -> Result<u64, &'static str> {
        let mut value = 0u64;
        let mut position = 0u8;

        loop {
            let byte = reader.read_byte()?;
            value |= u64::from(byte & Self::SEGMENT_BITS) << position;
            if (byte & Self::CONTINUE_BIT) == 0 {
                return Ok(value);
            }

            position += 7;

            if position >= 64 {
                return Err("VarLong is too large");
            }
        }
    }
    type Meta = ();
}

impl McProtoSelf for String {
    type Meta = Length;
    fn write(self, writer: &mut dyn McWriter) -> Result<(), &'static str> {
        VarNum::write(self.encode_utf16().count() as u32, writer)?;
        for byte in self.bytes() {
            byte.write(writer)?;
        }
        Ok(())
    }
    fn read(reader: &mut dyn McReader, _: Self::Meta) -> Result<Self, &'static str> {
        let length: u32 = VarNum::read(reader, ())?;
        let mut out = Vec::<u8>::new();
        let mut curr_length = 0u32;
        while curr_length < length {
            out.push(reader.read_byte()?);
            if let Ok(s) = str::from_utf8(out.as_slice()) {
                curr_length = s.encode_utf16().count() as u32;
            }
        }

        Ok(String::from_utf8(out).unwrap())
    }
}
