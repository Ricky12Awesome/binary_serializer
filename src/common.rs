use crate::encoder::{Serializer, Encoder};
use crate::decoder::{Deserializer, Decoder};
use std::hash::Hash;

pub struct MapEntry<K: Eq + Hash, V>(pub K, pub V);

impl <K: Serializer + Eq + Hash, V: Serializer> Serializer for MapEntry<&K, &V> {
  fn encode(&self, encoder: &mut impl Encoder) {
    self.0.encode(encoder);
    self.1.encode(encoder);
  }
}

impl <K: Deserializer + Eq + Hash, V: Deserializer> Deserializer for MapEntry<K, V> {
  fn decode(decoder: &mut impl Decoder) -> Self {

    MapEntry(decoder.decode_value::<K>(), decoder.decode_value::<V>())
  }
}