use std::hash::Hash;
use std::mem::size_of;

use crate::decoder::{Decoder, DecoderResult, Deserializer};
use crate::encoder::{Encoder, Serializer};

#[derive(Copy, Clone, Debug)]
pub enum ByteEndian {
  Big,
  Little,
}

impl ByteEndian {
  pub const fn is_native(&self) -> bool {
    match self {
      ByteEndian::Big => cfg!(target_endian = "big"),
      ByteEndian::Little => cfg!(target_endian = "little")
    }
  }
}

pub trait EndianValue: Sized {
  const SIZE: usize = size_of::<Self>();

  fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self;
  fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self;

  fn to_bytes_le(self) -> [u8; Self::SIZE];
  fn to_bytes_be(self) -> [u8; Self::SIZE];

  fn from_bytes_of(endian: ByteEndian, bytes: [u8; Self::SIZE]) -> Self {
    match endian {
      ByteEndian::Big => Self::from_bytes_be(bytes),
      ByteEndian::Little => Self::from_bytes_le(bytes)
    }
  }

  fn to_bytes_of(self, endian: ByteEndian) -> [u8; Self::SIZE] {
    match endian {
      ByteEndian::Big =>  self.to_bytes_be(),
      ByteEndian::Little => self.to_bytes_le(),
    }
  }
}

macro_rules! impl_from_endian {
  ($($type:ty),+ $(,)?) => {
    $(impl EndianValue for $type {
      fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self {
        Self::from_le_bytes(bytes)
      }

      fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
        Self::from_be_bytes(bytes)
      }

      fn to_bytes_le(self) -> [u8; Self::SIZE] {
        self.to_le_bytes()
      }

      fn to_bytes_be(self) -> [u8; Self::SIZE] {
        self.to_be_bytes()
      }
    })+
  };
}

impl_from_endian!(
  u8, u16, u32, u64, u128,
  i8, i16, i32, i64, i128,
  f32, f64
);

impl EndianValue for usize {
  const SIZE: usize = size_of::<u64>();

  fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self {
    u64::from_le_bytes(bytes) as usize
  }

  fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
    u64::from_be_bytes(bytes) as usize
  }

  fn to_bytes_le(self) -> [u8; Self::SIZE] {
    (self as u64).to_le_bytes()
  }

  fn to_bytes_be(self) -> [u8; Self::SIZE] {
    (self as u64).to_be_bytes()
  }
}

impl EndianValue for isize {
  const SIZE: usize = size_of::<i64>();

  fn from_bytes_le(bytes: [u8; Self::SIZE]) -> Self {
    i64::from_le_bytes(bytes) as isize
  }

  fn from_bytes_be(bytes: [u8; Self::SIZE]) -> Self {
    i64::from_be_bytes(bytes) as isize
  }

  fn to_bytes_le(self) -> [u8; Self::SIZE] {
    (self as i64).to_le_bytes()
  }

  fn to_bytes_be(self) -> [u8; Self::SIZE] {
    (self as i64).to_be_bytes()
  }
}

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