use std::any::type_name;
use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::hash::Hash;
use std::iter::FromIterator;
use std::mem::size_of;
use std::ops::Index;

use crate::v2::common::{ByteEndian, IS_LITTLE_ENDIAN, MapEntry};

pub type DecoderResult<T> = std::result::Result<T, DecoderError>;

#[derive(Debug)]
pub enum DecoderError {
  NotEnoughBytes {
    type_name: String,
    index: usize,
  },
  InvalidUTF16(std::string::FromUtf16Error),
}

impl DecoderError {
  pub fn not_enough_bytes(type_name: impl ToString, index: usize) -> Self {
    Self::NotEnoughBytes {
      type_name: type_name.to_string(),
      index,
    }
  }
}

impl Display for DecoderError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DecoderError::NotEnoughBytes { type_name, index } => {
        f.write_str(&format!("not enough bytes left to decode `{}` starting at index `{}`", type_name, index))
      }
      DecoderError::InvalidUTF16(err) => {
        Display::fmt(err, f)
      }
    }
  }
}

impl Error for DecoderError {}

pub trait Decoder: Sized {
  fn decode_u8(&mut self) -> DecoderResult<u8>;
  fn decode_u16(&mut self) -> DecoderResult<u16>;
  fn decode_u32(&mut self) -> DecoderResult<u32>;
  fn decode_u64(&mut self) -> DecoderResult<u64>;
  fn decode_u128(&mut self) -> DecoderResult<u128>;
  fn decode_usize(&mut self) -> DecoderResult<usize> { self.decode_u64().map(|it| it as usize) }

  fn decode_i8(&mut self) -> DecoderResult<i8>;
  fn decode_i16(&mut self) -> DecoderResult<i16>;
  fn decode_i32(&mut self) -> DecoderResult<i32>;
  fn decode_i64(&mut self) -> DecoderResult<i64>;
  fn decode_i128(&mut self) -> DecoderResult<i128>;
  fn decode_isize(&mut self) -> DecoderResult<isize> { self.decode_i64().map(|it| it as isize) }

  fn decode_f32(&mut self) -> DecoderResult<f32>;
  fn decode_f64(&mut self) -> DecoderResult<f64>;

  fn decode_bool(&mut self) -> DecoderResult<bool> { self.decode_u8().map(|it| it != 0) }

  fn decode_slice<T: Deserializer>(&mut self) -> DecoderResult<Vec<T>> {
    let len = self.decode_usize()?;
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
      vec.push(T::decode(self)?);
    }

    Ok(vec)
  }

  fn decode_string(&mut self) -> DecoderResult<String> {
    let data = self.decode_slice::<u16>()?;

    String::from_utf16(&data).map_err(DecoderError::InvalidUTF16)
  }

  fn decode_map<K: Deserializer + Eq + Hash, V: Deserializer>(&mut self) -> DecoderResult<HashMap<K, V>> {
    let entries = self.decode_slice::<MapEntry<K, V>>()?;
    let mut map = HashMap::with_capacity(entries.len());

    for entry in entries {
      map.insert(entry.0, entry.1);
    }

    Ok(map)
  }

  fn decode_value<T: Deserializer>(&mut self) -> DecoderResult<T> {
    T::decode(self)
  }
}

pub struct ByteDecoder<'a> {
  bytes: &'a [u8],
  endian: ByteEndian,
  index: usize,
}

impl<'a> ByteDecoder<'a> {
  pub fn new(bytes: &'a [u8], endian: ByteEndian) -> Self {
    Self { bytes, endian, index: 0 }
  }

  pub fn bytes(&self) -> &[u8] { &self.bytes }

  fn read_bytes<const N: usize>(&mut self, type_name: &str) -> DecoderResult<[u8; N]> {
    let value = self
      .bytes
      .get(self.index..self.index + N)
      .and_then(|bytes| bytes.try_into().ok())
      .ok_or_else(|| DecoderError::not_enough_bytes(type_name, self.index))?;

    self.index += N;

    Ok(value)
  }
}

impl<'a> Decoder for ByteDecoder<'a> {
  fn decode_u8(&mut self) -> DecoderResult<u8> { Ok(u8::from_le_bytes(self.read_bytes::<1>("u8")?)) }
  fn decode_u16(&mut self) -> DecoderResult<u16> { Ok(u16::from_le_bytes(self.read_bytes::<2>("u16")?)) }
  fn decode_u32(&mut self) -> DecoderResult<u32> { Ok(u32::from_le_bytes(self.read_bytes::<4>("u32")?)) }
  fn decode_u64(&mut self) -> DecoderResult<u64> { Ok(u64::from_le_bytes(self.read_bytes::<8>("u64")?)) }
  fn decode_u128(&mut self) -> DecoderResult<u128> { Ok(u128::from_le_bytes(self.read_bytes::<16>("u128")?)) }

  fn decode_i8(&mut self) -> DecoderResult<i8> { Ok(i8::from_le_bytes(self.read_bytes::<1>("i8")?)) }
  fn decode_i16(&mut self) -> DecoderResult<i16> { Ok(i16::from_le_bytes(self.read_bytes::<2>("i16")?)) }
  fn decode_i32(&mut self) -> DecoderResult<i32> { Ok(i32::from_le_bytes(self.read_bytes::<4>("i32")?)) }
  fn decode_i64(&mut self) -> DecoderResult<i64> { Ok(i64::from_le_bytes(self.read_bytes::<8>("i64")?)) }
  fn decode_i128(&mut self) -> DecoderResult<i128> { Ok(i128::from_le_bytes(self.read_bytes::<16>("i128")?)) }

  fn decode_f32(&mut self) -> DecoderResult<f32> { Ok(f32::from_le_bytes(self.read_bytes::<4>("f32")?)) }
  fn decode_f64(&mut self) -> DecoderResult<f64> { Ok(f64::from_le_bytes(self.read_bytes::<8>("f64")?)) }

  fn decode_slice<T: Deserializer>(&mut self) -> DecoderResult<Vec<T>> {
    let len = self.decode_usize()?;
    let mut vec = Vec::with_capacity(len);

    let begin = self.index;
    let end = begin + len * T::SIZE;

    if end > self.bytes.len() {
      return Err(DecoderError::not_enough_bytes(format!("[{}; {}]", type_name::<T>(), len), self.index));
    }

    // if self.endian.is_native() && T::IS_PRIMITIVE {
    //   unsafe {
    //     let ptr = self.bytes.get_unchecked(begin) as *const u8;
    //     let ptr = ptr as *const T;
    //
    //     for i in 0..len {
    //       vec.push(ptr.add(i).read());
    //     }
    //   }
    //
    //   return Ok(vec)
    // }

    for _ in 0..len {
      vec.push(T::decode(self)?);
    }

    Ok(vec)
  }
}

pub trait FromBytes: Deserializer + Sized {
  fn from_bytes(bytes: &[u8], endian: ByteEndian) -> DecoderResult<Self> {
    let mut decoder = ByteDecoder::new(bytes, endian);
    Ok(Self::decode(&mut decoder)?)
  }
}

impl<T: Deserializer> FromBytes for T {}

pub trait Deserializer: Sized {
  const IS_PRIMITIVE: bool = false;
  const SIZE: usize = size_of::<Self>();

  fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self>;
}

impl Deserializer for String {
  fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
    decoder.decode_string()
  }
}

impl<T: Deserializer> Deserializer for Vec<T> {
  fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
    decoder.decode_slice()
  }
}

impl<K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for HashMap<K, V> {
  fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
    decoder.decode_map()
  }
}

macro_rules! impl_deserializer_tuple {
  ($($name:ident),+) => {
    impl <$($name: Deserializer),+> Deserializer for ($($name),+) {
      const IS_PRIMITIVE: bool = $($name::IS_PRIMITIVE)&&+;

      fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
        Ok(($(decoder.decode_value::<$name>()?),+))
      }
    }
  };
}

impl_deserializer_tuple!(A, B);
impl_deserializer_tuple!(A, B, C);
impl_deserializer_tuple!(A, B, C, D);
impl_deserializer_tuple!(A, B, C, D, E);
impl_deserializer_tuple!(A, B, C, D, E, F);
impl_deserializer_tuple!(A, B, C, D, E, F, G);
impl_deserializer_tuple!(A, B, C, D, E, F, G, J);
impl_deserializer_tuple!(A, B, C, D, E, F, G, J, K);

macro_rules! impl_deserializer {
  ($(($type:ty, $decode:ident)),+ $(,)?) => {
    $(impl Deserializer for $type {
      const IS_PRIMITIVE: bool = true;

      fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
        decoder.$decode()
      }
    })+
  };
}

impl_deserializer!(
  (u8, decode_u8), (u16, decode_u16), (u32, decode_u32), (u64, decode_u64), (u128, decode_u128), (usize, decode_usize),
  (i8, decode_i8), (i16, decode_i16), (i32, decode_i32), (i64, decode_i64), (i128, decode_i128), (isize, decode_isize),
  (f32, decode_f32), (f64, decode_f64), (bool, decode_bool),
);

