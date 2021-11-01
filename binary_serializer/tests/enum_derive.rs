use binary_serializer::prelude::*;

#[derive(Debug, Serializer, Deserializer)]
enum Enum {
  A,
  B(u32, u32),
  C { x: u32, y: u32 },
}

#[test]
fn test_enum() {
  let bytes = Enum::B(2, 3).to_bytes(ByteEndian::Little);

  println!("{:?}", bytes);
  println!("{:?}", Enum::from_bytes(&bytes, ByteEndian::Little));
}
