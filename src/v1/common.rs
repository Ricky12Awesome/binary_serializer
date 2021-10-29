use std::hash::Hash;

use crate::v1::decoder::{Decoder, Deserializer};
use crate::v1::encoder::{Encoder, Serializer};

pub struct MapEntry<K: Eq + Hash, V>(pub K, pub V);

impl<K: Serializer + Eq + Hash, V: Serializer> Serializer for MapEntry<&K, &V> {
  fn encode(&self, encoder: &mut impl Encoder) {
    self.0.encode(encoder);
    self.1.encode(encoder);
  }
}

impl<K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for MapEntry<K, V> {
  fn decode(decoder: &mut impl Decoder) -> crate::v1::decoder::DecoderResult<Self> {
    Ok(MapEntry(decoder.decode_value::<K>()?, decoder.decode_value::<V>()?))
  }
}