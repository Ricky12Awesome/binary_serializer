#![allow(dead_code)]

mod encoder {
  use std::io::Write;

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
    fn encode_value<T: Serializer>(&mut self, value: impl Serializer);
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
        panic!("All elements must be the same size.")
      };

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

    fn encode_value<T: Serializer>(&mut self, value: impl Serializer) {
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
}

mod decoder {
  pub trait Decoder {
    fn decode_u8(&mut self) -> u8;
    fn decode_u16(&mut self) -> u16;
    fn decode_u32(&mut self) -> u32;
    fn decode_u64(&mut self) -> u64;
    fn decode_u128(&mut self) -> u128;
    fn decode_usize(&mut self) -> usize { self.decode_u64() as usize }

    fn decode_i8(&mut self) -> i8;
    fn decode_i16(&mut self) -> i16;
    fn decode_i32(&mut self) -> i32;
    fn decode_i64(&mut self) -> i64;
    fn decode_i128(&mut self) -> i128;
    fn decode_isize(&mut self) -> isize { self.decode_i64() as isize }

    fn decode_f32(&mut self) -> f32;
    fn decode_f64(&mut self) -> f64;

    fn decode_string(&mut self) -> String;
    fn decode_slice<T: Deserializer>(&mut self) -> Vec<T>;
    fn decode_value<T: Deserializer>(&mut self) -> T;
  }

  pub struct ByteDecoder {
    bytes: Vec<u8>,
    index: usize,
  }

  impl ByteDecoder {
    pub fn new(bytes: Vec<u8>) -> Self {
      Self { bytes, index: 0 }
    }

    pub fn bytes(&self) -> &Vec<u8> {
      &self.bytes
    }

    fn next_bytes_n<const N: usize>(&mut self) -> [u8; N] {
      let bytes = &self.bytes[self.index..self.index + N];

      self.index += N;

      unsafe {
        *(bytes.as_ptr() as *const [u8; N])
      }
    }
  }

  impl Decoder for ByteDecoder {
    fn decode_u8(&mut self) -> u8 { u8::from_ne_bytes(self.next_bytes_n::<1>()) }
    fn decode_u16(&mut self) -> u16 { u16::from_ne_bytes(self.next_bytes_n::<2>()) }
    fn decode_u32(&mut self) -> u32 { u32::from_ne_bytes(self.next_bytes_n::<4>()) }
    fn decode_u64(&mut self) -> u64 { u64::from_ne_bytes(self.next_bytes_n::<8>()) }
    fn decode_u128(&mut self) -> u128 { u128::from_ne_bytes(self.next_bytes_n::<16>()) }

    fn decode_i8(&mut self) -> i8 { i8::from_ne_bytes(self.next_bytes_n::<1>()) }
    fn decode_i16(&mut self) -> i16 { i16::from_ne_bytes(self.next_bytes_n::<2>()) }
    fn decode_i32(&mut self) -> i32 { i32::from_ne_bytes(self.next_bytes_n::<4>()) }
    fn decode_i64(&mut self) -> i64 { i64::from_ne_bytes(self.next_bytes_n::<8>()) }
    fn decode_i128(&mut self) -> i128 { i128::from_ne_bytes(self.next_bytes_n::<16>()) }

    fn decode_f32(&mut self) -> f32 { f32::from_ne_bytes(self.next_bytes_n::<4>()) }
    fn decode_f64(&mut self) -> f64 { f64::from_ne_bytes(self.next_bytes_n::<8>()) }

    fn decode_string(&mut self) -> String {
      let data = self.decode_slice::<u16>();

      String::from_utf16(&data).unwrap()
    }

    fn decode_slice<T: Deserializer>(&mut self) -> Vec<T> {
      let total_bytes = self.decode_usize();
      let bytes_per_element = self.decode_usize();
      let len = total_bytes / bytes_per_element;
      let mut vec = Vec::with_capacity(len);

      for _ in 0..len {
        vec.push(T::decode(self));
      }

      vec
    }

    fn decode_value<T: Deserializer>(&mut self) -> T {
      T::decode(self)
    }
  }

  pub trait FromBytes: Deserializer + Sized {
    fn from_bytes(bytes: Vec<u8>) -> Self {
      let mut decoder = ByteDecoder::new(bytes);
      Self::decode(&mut decoder)
    }
  }

  impl<T: Deserializer> FromBytes for T {}

  pub trait Deserializer {
    fn decode(decoder: &mut impl Decoder) -> Self;
  }

  impl<T: Deserializer> Deserializer for Vec<T> {
    fn decode(decoder: &mut impl Decoder) -> Self {
      decoder.decode_slice()
    }
  }

  macro_rules! impl_deserializer {
    ($(($type:ty, $decode:ident)),+ $(,)?) => {
      $(impl Deserializer for $type {
        fn decode(decoder: &mut impl Decoder) -> Self {
          decoder.$decode()
        }
      })+
    };
  }

  impl_deserializer!(
    (u8, decode_u8), (u16, decode_u16), (u32, decode_u32), (u64, decode_u64), (u128, decode_u128), (usize, decode_usize),
    (i8, decode_i8), (i16, decode_i16), (i32, decode_i32), (i64, decode_i64), (i128, decode_i128), (isize, decode_isize),
    (f32, decode_f32), (f64, decode_f64), (String, decode_string)
  );
}

#[derive(Debug, Eq, PartialEq)]
struct Data {
  name: String,
  age: u32,
  some_random_data: Vec<u32>,
}

impl Data {
  pub fn new(name: impl ToString, age: u32, some_random_data: Vec<u32>) -> Self {
    Self { name: name.to_string(), age, some_random_data }
  }
}

impl encoder::Serializer for Data {
  fn encode(&self, encoder: &mut impl encoder::Encoder) {
    encoder.encode_string(&self.name);
    encoder.encode_u32(self.age);
    encoder.encode_slice(&self.some_random_data);
  }
}

impl decoder::Deserializer for Data {
  fn decode(decoder: &mut impl decoder::Decoder) -> Self {
    Self {
      name: decoder.decode_string(),
      age: decoder.decode_u32(),
      some_random_data: decoder.decode_slice::<u32>(),
    }
  }
}

#[cfg(test)]
#[test]
fn run() {
  use encoder::*;
  use decoder::*;

  let data = Data::new("Some Name", 69, vec![23, 32, 52, 542, 42, 643]);

  let start = std::time::SystemTime::now();
  let bytes = data.to_bytes();
  let data2 = Data::from_bytes(bytes);
  let end = start.elapsed().unwrap();

  println!("{:?}", end);
  assert_eq!(data, data2);
}