use std::collections::HashMap;
use crate::common::MapEntry;
use std::hash::Hash;

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

  fn decode_bool(&mut self) -> bool {
    self.decode_u8() != 0
  }

  fn decode_string(&mut self) -> String;
  fn decode_slice<T: Deserializer>(&mut self) -> Vec<T>;
  fn decode_map<K: Deserializer + Eq + Hash, V: Deserializer>(&mut self) -> HashMap<K, V>;
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

    if total_bytes == 0 || bytes_per_element == 0 {
      return Vec::new()
    }

    let len = total_bytes / bytes_per_element;
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
      vec.push(T::decode(self));
    }

    vec
  }

  fn decode_map<K: Deserializer + Eq + Hash, V: Deserializer>(&mut self) -> HashMap<K, V> {
    let mut map = HashMap::new();

    for entry in self.decode_slice::<MapEntry<K, V>>() {
      map.insert(entry.0, entry.1);
    }

    map
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

impl <K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for HashMap<K, V> {
  fn decode(decoder: &mut impl Decoder) -> Self {
    decoder.decode_map()
  }
}

macro_rules! impl_deserializer_tuple {
  ($($name:ident),+) => {
    impl <$($name: Deserializer),+> Deserializer for ($($name),+) {
      fn decode(decoder: &mut impl Decoder) -> Self {
        ($(decoder.decode_value::<$name>()),+)
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
      fn decode(decoder: &mut impl Decoder) -> Self {
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

