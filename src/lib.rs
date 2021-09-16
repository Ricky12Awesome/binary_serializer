#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use crate::encoder::*;
use crate::decoder::*;
use std::iter::FromIterator;


pub mod common;
pub mod encoder;
pub mod decoder;

#[derive(Debug, Eq, PartialEq)]
struct Data {
  name: String,
  age: u32,
  some_random_data: Vec<HashMap<String, u32>>,
  non_string_map: HashMap<u32, u32>,
}

impl encoder::Serializer for Data {
  fn encode(&self, encoder: &mut impl encoder::Encoder) {
    encoder.encode_string(&self.name);
    encoder.encode_u32(self.age);
    encoder.encode_slice(&self.some_random_data);
    encoder.encode_map(&self.non_string_map);
  }
}

impl decoder::Deserializer for Data {
  fn decode(decoder: &mut impl decoder::Decoder) -> Self {
    Self {
      name: decoder.decode_string(),
      age: decoder.decode_u32(),
      some_random_data: decoder.decode_slice::<HashMap<String, u32>>(),
      non_string_map: decoder.decode_map::<u32, u32>(),
    }
  }
}

#[cfg(test)]
#[test]
fn run() {
  let data = Data {
    name: "Some Name".to_string(),
    age: 69,
    some_random_data: vec![
      HashMap::from_iter([("test".to_string(), 2)]),
      HashMap::from_iter([("eiuhgieurohbn".to_string(), 238)]),
      HashMap::from_iter([("".to_string(), 0)]),
    ],
    non_string_map: HashMap::from_iter([
      (0, 2934),
      (1, 39848953),
      (3, 340985),
    ])
  };

  let start = std::time::SystemTime::now();
  let bytes = data.to_bytes();
  let data2 = Data::from_bytes(bytes);
  let end = start.elapsed().unwrap();

  println!("{:?}", data);
  println!("{:?}", data2);
  println!("{:?}", end);
  assert_eq!(data, data2);
}
