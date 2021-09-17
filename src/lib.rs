#![allow(dead_code, unused_imports)]

use std::collections::HashMap;
use std::iter::FromIterator;

use crate::decoder::*;
use crate::encoder::*;

pub mod common;
pub mod encoder;
pub mod decoder;

#[derive(Debug, Eq, PartialEq)]
struct Data {
  name: String,
  age: u32,
  bool: bool,
  some_random_data: Vec<HashMap<String, u32>>,
  non_string_map: HashMap<u32, (u32, u32)>,
}

impl Serializer for Data {
  fn encode(&self, encoder: &mut impl Encoder) {
    encoder.encode_string(&self.name);
    encoder.encode_u32(self.age);
    encoder.encode_bool(self.bool);
    encoder.encode_slice(&self.some_random_data);
    encoder.encode_map(&self.non_string_map);
  }
}

impl Deserializer for Data {
  fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self> {
    Ok(Self {
      name: decoder.decode_string()?,
      age: decoder.decode_u32()?,
      bool: decoder.decode_bool()?,
      some_random_data: decoder.decode_slice::<HashMap<String, u32>>()?,
      non_string_map: decoder.decode_map::<u32, (u32, u32)>()?,
    })
  }
}

#[cfg(test)]
#[test]
fn test_invalid_data() {
  fn test<const N: usize>() {
    let bytes = [0x00; N];

    let value = Data::from_bytes(bytes);

    println!("{}: {:?}", N, value);
  }

  test::<16>();
  test::<32>();
  test::<64>();
  test::<96>();
  test::<128>();
  test::<192>();
  test::<256>();
}

#[cfg(test)]
#[test]
fn run() {
  let data = Data {
    name: "Some Name".to_string(),
    age: 69,
    bool: true,
    some_random_data: vec![
      HashMap::from_iter([("test".to_string(), 2)]),
      HashMap::from_iter([("eiuhgieurohbn".to_string(), 238)]),
      HashMap::from_iter([("".to_string(), 0)]),
    ],
    non_string_map: HashMap::from_iter([
      (0, (2934, 32465346)),
      (1, (30945, 39846)),
      (2, (32495, 3453456)),
      (3, (945745, 32095)),
    ]),
  };

  let start = std::time::SystemTime::now();
  let bytes = data.to_bytes();
  let data2 = Data::from_bytes(bytes.clone()).unwrap();
  let end = start.elapsed().unwrap();

  println!("Serialized Bytes {:?}", &bytes);
  println!("Original {:?}", data);
  println!("Deserialized {:?}", data2);
  println!("Serialize to Deserialize took {:?}", end);
  assert_eq!(data, data2);
}
