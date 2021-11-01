use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;

use crate::common::{ByteEndian, EndianValue, MapEntry};

pub trait Encoder: Sized {
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

  fn encode_bool(&mut self, value: bool) {
    self.encode_u8(value as u8);
  }

  fn encode_slice<T: Serializer>(&mut self, value: &[T]) {
    self.encode_usize(value.len());

    for value in value {
      value.encode(self);
    }
  }

  fn encode_string(&mut self, value: impl ToString) {
    let str = value.to_string();
    let vec = str.encode_utf16().collect::<Vec<_>>();

    self.encode_slice(&vec);
  }

  fn encode_map<K: Serializer + Eq + Hash, V: Serializer>(&mut self, value: &HashMap<K, V>) {
    let values = value
      .iter()
      .map(|it| MapEntry(it.0, it.1))
      .collect::<Vec<_>>();

    self.encode_slice(&values);
  }

  fn encode_value<T: Serializer>(&mut self, value: &T) {
    value.encode(self);
  }
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
  endian: ByteEndian,
}

impl ByteEncoder {
  pub fn new(endian: ByteEndian) -> Self {
    Self {
      bytes: vec![],
      endian,
    }
  }

  pub fn bytes(&self) -> &Vec<u8> {
    &self.bytes
  }

  fn write<T: EndianValue<SIZE>, const SIZE: usize>(&mut self, value: T) {
    self.bytes.write(&value.to_bytes_of(self.endian)).unwrap();
  }
}

impl Encoder for ByteEncoder {
  fn encode_u8(&mut self, value: u8) { self.write(value); }
  fn encode_u16(&mut self, value: u16) { self.write(value); }
  fn encode_u32(&mut self, value: u32) { self.write(value); }
  fn encode_u64(&mut self, value: u64) { self.write(value); }
  fn encode_u128(&mut self, value: u128) { self.write(value) }

  fn encode_i8(&mut self, value: i8) { self.write(value); }
  fn encode_i16(&mut self, value: i16) { self.write(value); }
  fn encode_i32(&mut self, value: i32) { self.write(value); }
  fn encode_i64(&mut self, value: i64) { self.write(value); }
  fn encode_i128(&mut self, value: i128) { self.write(value) }

  fn encode_f32(&mut self, value: f32) { self.write(value); }
  fn encode_f64(&mut self, value: f64) { self.write(value); }
}

pub trait ToBytes: Serializer {
  fn to_bytes(&self, endian: ByteEndian) -> Vec<u8> {
    let mut encoder = ByteEncoder::new(endian);
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
  (f32, encode_f32), (f64, encode_f64), (bool, encode_bool)
);

