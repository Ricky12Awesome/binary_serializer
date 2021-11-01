use std::hash::Hash;

use crate::decoder::{Decoder, DecoderResult, Deserializer};
use crate::encoder::{Encoder, Serializer};

#[derive(Copy, Clone, Debug)]
pub enum ByteEndian {
  Big,
  Little,
}

impl ByteEndian {
  #[cfg(target_endian = "little")]
  const NATIVE: Self = ByteEndian::Little;
  #[cfg(target_endian = "big")]
  const NATIVE: Self = ByteEndian::Big;

  pub const fn is_native(&self) -> bool {
    match self {
      ByteEndian::Big => cfg!(target_endian = "big"),
      ByteEndian::Little => cfg!(target_endian = "little")
    }
  }
}

pub trait EndianValue<const SIZE: usize>: Sized {
  fn from_bytes_le(bytes: [u8; SIZE]) -> Self;
  fn from_bytes_be(bytes: [u8; SIZE]) -> Self;

  fn to_bytes_le(self) -> [u8; SIZE];
  fn to_bytes_be(self) -> [u8; SIZE];

  fn from_bytes_of(endian: ByteEndian, bytes: [u8; SIZE]) -> Self {
    match endian {
      ByteEndian::Big => Self::from_bytes_be(bytes),
      ByteEndian::Little => Self::from_bytes_le(bytes)
    }
  }

  fn to_bytes_of(self, endian: ByteEndian) -> [u8; SIZE] {
    match endian {
      ByteEndian::Big => self.to_bytes_be(),
      ByteEndian::Little => self.to_bytes_le(),
    }
  }
}

macro_rules! impl_from_endian {
  ($(($type:ty, $size:literal)),+ $(,)?) => {
    $(impl EndianValue<$size> for $type {
      fn from_bytes_le(bytes: [u8; $size]) -> Self {
        Self::from_le_bytes(bytes)
      }

      fn from_bytes_be(bytes: [u8; $size]) -> Self {
        Self::from_be_bytes(bytes)
      }

      fn to_bytes_le(self) -> [u8; $size] {
        self.to_le_bytes()
      }

      fn to_bytes_be(self) -> [u8; $size] {
        self.to_be_bytes()
      }
    })+
  };
}

impl_from_endian!(
  (u8, 1), (u16, 2), (u32, 4), (u64, 8), (u128, 16), (usize, 8),
  (i8, 1), (i16, 2), (i32, 4), (i64, 8), (i128, 16), (isize, 8),
  (f32, 4), (f64, 8)
);

pub struct MapEntry<K: Eq + Hash, V>(pub K, pub V);

impl<K: Serializer + Eq + Hash, V: Serializer> Serializer for MapEntry<&K, &V> {
  fn encode(&self, encoder: &mut impl Encoder) {
    self.0.encode(encoder);
    self.1.encode(encoder);
  }
}

impl<K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for MapEntry<K, V> {
  fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
    Ok(MapEntry(decoder.decode_value::<K>()?, decoder.decode_value::<V>()?))
  }
}