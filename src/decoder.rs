use std::collections::HashMap;
use crate::common::MapEntry;
use std::hash::Hash;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Decoder: Sized {
  fn decode_u8(&mut self) -> Result<u8>;
  fn decode_u16(&mut self) -> Result<u16>;
  fn decode_u32(&mut self) -> Result<u32>;
  fn decode_u64(&mut self) -> Result<u64>;
  fn decode_u128(&mut self) -> Result<u128>;
  fn decode_usize(&mut self) -> Result<usize> { self.decode_u64().map(|it| it as usize) }

  fn decode_i8(&mut self) -> Result<i8>;
  fn decode_i16(&mut self) -> Result<i16>;
  fn decode_i32(&mut self) -> Result<i32>;
  fn decode_i64(&mut self) -> Result<i64>;
  fn decode_i128(&mut self) -> Result<i128>;
  fn decode_isize(&mut self) -> Result<isize> { self.decode_i64().map(|it| it as isize) }

  fn decode_f32(&mut self) -> Result<f32>;
  fn decode_f64(&mut self) -> Result<f64>;

  fn decode_bool(&mut self) -> Result<bool> { self.decode_u8().map(|it| it != 0) }

  fn decode_slice<T: Deserializer>(&mut self) -> Result<Vec<T>>;

  fn decode_string(&mut self) -> Result<String> {
    let data = self.decode_slice::<u16>()?;

    Ok(String::from_utf16(&data)?)
  }

  fn decode_map<K: Deserializer + Eq + Hash, V: Deserializer>(&mut self) -> Result<HashMap<K, V>> {
    let mut map = HashMap::new();

    for entry in self.decode_slice::<MapEntry<K, V>>()? {
      map.insert(entry.0, entry.1);
    }

    Ok(map)
  }

  fn decode_value<T: Deserializer>(&mut self) -> Result<T> {
    T::decode(self)
  }
}

pub struct ByteDecoder {
  bytes: Vec<u8>,
  index: usize,
}

impl ByteDecoder {
  pub fn new(bytes: impl IntoIterator<Item = u8>) -> Self {
    Self { bytes: bytes.into_iter().collect(), index: 0 }
  }

  pub fn bytes(&self) -> &Vec<u8> {
    &self.bytes
  }

  fn next_bytes_n<const N: usize>(&mut self, typ: &str) -> Result<[u8; N]> {
    let bytes = &self.bytes
      .get(self.index..self.index + N)
      .ok_or(format!("not enough bytes left to decode `{}` starting at index `{}`", typ, self.index))?;

    self.index += N;

    let value = unsafe {
      *(bytes.as_ptr() as *const [u8; N])
    };

    Ok(value)
  }
}

impl Decoder for ByteDecoder {
  fn decode_u8(&mut self) -> Result<u8> { Ok(u8::from_ne_bytes(self.next_bytes_n::<1>("u8")?)) }
  fn decode_u16(&mut self) -> Result<u16> { Ok(u16::from_ne_bytes(self.next_bytes_n::<2>("u16")?)) }
  fn decode_u32(&mut self) -> Result<u32> { Ok(u32::from_ne_bytes(self.next_bytes_n::<4>("u32")?)) }
  fn decode_u64(&mut self) -> Result<u64> { Ok(u64::from_ne_bytes(self.next_bytes_n::<8>("u64")?)) }
  fn decode_u128(&mut self) -> Result<u128> { Ok(u128::from_ne_bytes(self.next_bytes_n::<16>("u128")?)) }

  fn decode_i8(&mut self) -> Result<i8> { Ok(i8::from_ne_bytes(self.next_bytes_n::<1>("i8")?)) }
  fn decode_i16(&mut self) -> Result<i16> { Ok(i16::from_ne_bytes(self.next_bytes_n::<2>("i16")?)) }
  fn decode_i32(&mut self) -> Result<i32> { Ok(i32::from_ne_bytes(self.next_bytes_n::<4>("i32")?)) }
  fn decode_i64(&mut self) -> Result<i64> { Ok(i64::from_ne_bytes(self.next_bytes_n::<8>("i64")?)) }
  fn decode_i128(&mut self) -> Result<i128> { Ok(i128::from_ne_bytes(self.next_bytes_n::<16>("i128")?)) }

  fn decode_f32(&mut self) -> Result<f32> { Ok(f32::from_ne_bytes(self.next_bytes_n::<4>("f32")?)) }
  fn decode_f64(&mut self) -> Result<f64> { Ok(f64::from_ne_bytes(self.next_bytes_n::<8>("f64")?)) }

  fn decode_slice<T: Deserializer>(&mut self) -> Result<Vec<T>> {
    let total_bytes = self.decode_usize()?;
    let bytes_per_element = self.decode_usize()?;

    if total_bytes == 0 || bytes_per_element == 0 {
      return Ok(Vec::new())
    }

    let len = total_bytes / bytes_per_element;
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
      vec.push(T::decode(self)?);
    }

    Ok(vec)
  }
}

pub trait FromBytes: Deserializer + Sized {
  fn from_bytes(bytes: impl IntoIterator<Item = u8>) -> Result<Self> {
    let mut decoder = ByteDecoder::new(bytes);
    Ok(Self::decode(&mut decoder)?)
  }
}

impl<T: Deserializer> FromBytes for T {}

pub trait Deserializer: Sized {
  fn decode(decoder: &mut impl Decoder) -> Result<Self>;
}

impl<T: Deserializer> Deserializer for Vec<T> {
  fn decode(decoder: &mut impl Decoder) -> Result<Self> {
    decoder.decode_slice()
  }
}

impl <K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for HashMap<K, V> {
  fn decode(decoder: &mut impl Decoder) -> Result<Self> {
    decoder.decode_map()
  }
}

macro_rules! impl_deserializer_tuple {
  ($($name:ident),+) => {
    impl <$($name: Deserializer),+> Deserializer for ($($name),+) {
      fn decode(decoder: &mut impl Decoder) -> Result<Self> {
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
      fn decode(decoder: &mut impl Decoder) -> Result<Self> {
        decoder.$decode()
      }
    })+
  };
}

impl_deserializer!(
  (u8, decode_u8), (u16, decode_u16), (u32, decode_u32), (u64, decode_u64), (u128, decode_u128), (usize, decode_usize),
  (i8, decode_i8), (i16, decode_i16), (i32, decode_i32), (i64, decode_i64), (i128, decode_i128), (isize, decode_isize),
  (f32, decode_f32), (f64, decode_f64), (bool, decode_bool), (String, decode_string)
);

