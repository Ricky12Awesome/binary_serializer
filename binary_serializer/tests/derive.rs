pub use binary_serializer::prelude::*;

use std::fmt::Debug;

#[derive(Debug, PartialEq, Serializer, Deserializer)]
struct Unit;

#[derive(Debug, PartialEq, Serializer, Deserializer)]
struct Tuple(u32, u32);

#[derive(Debug, PartialEq, Serializer, Deserializer)]
struct Fields {
  unit: Unit,
  tuple: Tuple,
  enums: Vec<Enum>,
}

#[derive(Debug, PartialEq, Serializer, Deserializer)]
enum Enum {
  Unit,
  Tuple(u32, u32),
  Struct { x: u32, y: u32 },
}

fn test_valid<T: Serializer + Deserializer + PartialEq + Debug>(source: T) {
  let bytes = source.to_bytes(ByteEndian::Little);
  let parsed_le = T::from_bytes(&bytes, ByteEndian::Little);

  let bytes = source.to_bytes(ByteEndian::Big);
  let parsed_be = T::from_bytes(&bytes, ByteEndian::Big);

  let source = Ok(source);

  assert_eq!(source, parsed_le);
  assert_eq!(source, parsed_be);
}

fn test_invalid<T: Serializer + Deserializer + PartialEq + Debug>(source: T) {
  let bytes = source.to_bytes(ByteEndian::Big);
  let parsed_le = T::from_bytes(&bytes, ByteEndian::Little);

  let bytes = source.to_bytes(ByteEndian::Little);
  let parsed_be = T::from_bytes(&bytes, ByteEndian::Big);

  let source = Ok(source);

  assert_ne!(source, parsed_le);
  assert_ne!(source, parsed_be);
}

#[test]
fn valid() {
  test_valid(Enum::Unit);
  test_valid(Enum::Tuple(69, 420));
  test_valid(Enum::Struct { x: 69, y: 420 });

  test_valid(Unit);
  test_valid(Tuple(69, 420));
  test_valid(Fields {
    unit: Unit,
    tuple: Tuple(69, 420),
    enums: vec![Enum::Tuple(69, 420), Enum::Struct { x: 69, y: 420 }],
  });
}

#[test]
fn invalid() {
  test_invalid(Enum::Tuple(69, 420));
  test_invalid(Enum::Struct { x: 69, y: 420 });
  test_invalid(Tuple(69, 420));
  test_invalid(Fields {
    unit: Unit,
    tuple: Tuple(69, 420),
    enums: vec![Enum::Tuple(69, 420), Enum::Struct { x: 69, y: 420 }],
  });
}
