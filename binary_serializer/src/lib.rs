use crate::common::*;
use crate::decoder::*;
use crate::encoder::*;

pub mod common;
pub mod encoder;
pub mod decoder;

#[cfg(feature = "prelude")]
pub mod prelude {
  pub use crate::common::*;
  pub use crate::decoder::*;
  pub use crate::encoder::*;
}

#[derive(Debug, Eq, PartialEq, Serializer, Deserializer)]
struct Data {
  text: String,
  number: u32,
  list: Vec<u8>,
}

#[cfg(test)]
#[test]
fn run() {
  let data = Data {
    text: "Text".to_string(),
    number: 69,
    list: vec![4, 2, 0]
  };

  let bytes = data.to_bytes(ByteEndian::Little);
  let data2 = Data::from_bytes(&bytes, ByteEndian::Little).unwrap();

  println!("Serialized Bytes {:?}", &bytes);
  println!("Original {:?}", data);
  println!("Deserialized {:?}", data2);
  assert_eq!(data, data2);
}
