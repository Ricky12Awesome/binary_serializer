use std::hash::Hash;

use crate::v2::decoder::{Decoder, Deserializer};
use crate::v2::encoder::{Encoder, Serializer};

pub struct MapEntry<K: Eq + Hash, V>(pub K, pub V);

#[cfg(target_endian = "little")]
pub const IS_LITTLE_ENDIAN: bool = true;
#[cfg(not(target_endian = "little"))]
pub const IS_LITTLE_ENDIAN: bool = false;

impl<K: Serializer + Eq + Hash, V: Serializer> Serializer for MapEntry<&K, &V> {
  fn encode(&self, encoder: &mut impl Encoder) {
    self.0.encode(encoder);
    self.1.encode(encoder);
  }
}

impl<K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for MapEntry<K, V> {
  fn decode(decoder: &mut impl Decoder) -> crate::v2::decoder::DecoderResult<Self> {
    Ok(MapEntry(decoder.decode_value::<K>()?, decoder.decode_value::<V>()?))
  }
}