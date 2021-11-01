pub use binary_serializer::prelude::*;


#[derive(Default, Serializer, Deserializer)]
struct Unit;

#[derive(Default, Serializer, Deserializer)]
struct Tuple(u32, u32);

#[derive(Default, Serializer, Deserializer)]
struct Fields {
  unit: Unit,
  tuple: Tuple,
}

#[test]
fn test() {
  let bytes = Fields::default().to_bytes(ByteEndian::Little);

}