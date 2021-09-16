use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;

use crate::common::MapEntry;

pub trait Encoder {
  fn encode_u8(&mut self, value: u8);
  fn encode_u16(&mut self, value: u16);
  fn encode_u32(&mut self, value: u32);
  fn encode_u64(&mut self, value: u64);
  fn encode_u128(&mut self, value: u128);
  fn encode_usize(&mut self, value: usize) { self.encode_u64(value as u64); }

  fn encode_i8(&mut self, value: i8);
  fn encode_i16(&mut self, value: i16);
  fn encode_i32(&mut self, value: i32);
  fn encode_i64(&mut self, value: i64);
  fn encode_i128(&mut self, value: i128);
  fn encode_isize(&mut self, value: isize) { self.encode_i64(value as i64); }

  fn encode_f32(&mut self, value: f32);
  fn encode_f64(&mut self, value: f64);

  fn encode_string(&mut self, value: impl ToString);

  fn encode_slice<T: Serializer>(&mut self, value: &[T]);
  fn encode_map<K: Serializer + Eq + Hash, V: Serializer>(&mut self, value: &HashMap<K, V>);
  fn encode_value<T: Serializer>(&mut self, value: &impl Serializer);
}

pub struct ByteTracker {
  pub start: usize,
}

impl ByteTracker {
  pub fn begin(bytes: &Vec<u8>) -> Self {
    ByteTracker {
      start: bytes.len()
    }
  }

  pub fn end(&self, new_bytes: &Vec<u8>) -> usize {
    new_bytes.len() - self.start
  }
}

pub struct ByteEncoder {
  bytes: Vec<u8>,
}

impl ByteEncoder {
  pub fn new() -> Self {
    Self {
      bytes: vec![],
    }
  }

  pub fn bytes(&self) -> &Vec<u8> {
    &self.bytes
  }
}

impl Encoder for ByteEncoder {
  fn encode_u8(&mut self, value: u8) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_u16(&mut self, value: u16) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_u32(&mut self, value: u32) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_u64(&mut self, value: u64) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_u128(&mut self, value: u128) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }

  fn encode_i8(&mut self, value: i8) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_i16(&mut self, value: i16) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_i32(&mut self, value: i32) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_i64(&mut self, value: i64) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_i128(&mut self, value: i128) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }

  fn encode_f32(&mut self, value: f32) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }
  fn encode_f64(&mut self, value: f64) { self.bytes.write(&value.to_ne_bytes()).unwrap(); }

  fn encode_string(&mut self, value: impl ToString) {
    let str = value.to_string();
    let vec = str.encode_utf16().collect::<Vec<_>>();

    self.encode_slice(&vec);
  }

  fn encode_slice<T: Serializer>(&mut self, value: &[T]) {
    let total_bytes = ByteTracker::begin(&self.bytes);

    for value in value {
      value.encode(self);
    }

    let index = total_bytes.start;
    let total_bytes = total_bytes.end(&self.bytes);
    let bytes_per_element = total_bytes.checked_div(value.len());
    let bytes_per_element = if let Some(n) = bytes_per_element { n } else {
      if total_bytes != 0 {
        panic!("All elements must be the same size.")
      } else { 0 }
    };

    // TODO: Put this into separate function
    let mut bytes = bytes_per_element.to_ne_bytes();

    bytes.reverse();

    for b in bytes {
      self.bytes.insert(index, b);
    }

    let mut bytes = total_bytes.to_ne_bytes();

    bytes.reverse();

    for b in bytes {
      self.bytes.insert(index, b);
    }
  }

  fn encode_map<K: Serializer + Eq + Hash, V: Serializer>(&mut self, value: &HashMap<K, V>) {
    let values = value
      .iter()
      .map(|it| MapEntry(it.0, it.1))
      .collect::<Vec<_>>();

    self.encode_slice(&values);
  }

  fn encode_value<T: Serializer>(&mut self, value: &impl Serializer) {
    value.encode(self);
  }
}

pub trait ToBytes: Serializer {
  fn to_bytes(&self) -> Vec<u8> {
    let mut encoder = ByteEncoder::new();
    self.encode(&mut encoder);

    encoder.bytes
  }
}

impl<T: Serializer> ToBytes for T {}

pub trait Serializer {
  fn encode(&self, encoder: &mut impl Encoder);
}

impl Serializer for &str {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_string(self)
  }
}

impl Serializer for String {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_string(self)
  }
}

impl<T: Serializer> Serializer for &[T] {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_slice(self)
  }
}

impl<T: Serializer> Serializer for [T] {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_slice(self)
  }
}

impl<T: Serializer, const N: usize> Serializer for [T; N] {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_slice(self)
  }
}

impl<K: Serializer + Eq + Hash, V: Serializer> Serializer for HashMap<K, V> {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_map(self);
  }
}

macro_rules! impl_serializer_tuple {
  ($($name:ident),+) => {
    impl <$($name: Serializer),+> Serializer for ($($name),+) {
      #[allow(non_snake_case)]
      fn encode(&self, encoder: &mut impl Encoder) {
        let ($($name),+) = self;
        $($name.encode(encoder);)+
      }
    }
  };
}

impl_serializer_tuple!(A, B);
impl_serializer_tuple!(A, B, C);
impl_serializer_tuple!(A, B, C, D);
impl_serializer_tuple!(A, B, C, D, E);
impl_serializer_tuple!(A, B, C, D, E, F);
impl_serializer_tuple!(A, B, C, D, E, F, G);
impl_serializer_tuple!(A, B, C, D, E, F, G, J);
impl_serializer_tuple!(A, B, C, D, E, F, G, J, K);

macro_rules! impl_serializer {
  ($(($type:ty, $encode:ident)),+ $(,)?) => {
    $(impl Serializer for $type {
      fn encode(&self, encoder: &mut impl Encoder) {
        encoder.$encode(*self);
      }
    })+
  };
}

impl_serializer!(
  (u8, encode_u8), (u16, encode_u16), (u32, encode_u32), (u64, encode_u64), (u128, encode_u128), (usize, encode_usize),
  (i8, encode_i8), (i16, encode_i16), (i32, encode_i32), (i64, encode_i64), (i128, encode_i128), (isize, encode_isize),
  (f32, encode_f32), (f64, encode_f64)
);

