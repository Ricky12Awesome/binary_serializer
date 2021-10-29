#[cfg(not(feature = "exclude_v1"))]
pub mod v1;
#[cfg(feature = "v2")]
pub mod v2;

#[cfg(not(feature = "v2"))]
pub use v1::*;
#[cfg(feature = "v2")]
pub use v2::*;

#[cfg(test)]
mod v1_test {
  use crate::v1::encoder::ToBytes;
  use crate::v1::decoder::FromBytes;

  #[test]
  fn test_slice() {
    let source = [69u32; 10];
    println!("Source: {:?}", source);

    let bytes = source.to_bytes();
    let data = Vec::<u32>::from_bytes(bytes);
    println!("From: {:?}", data);

    if let Ok(data) = data {
      assert_eq!(source, data.as_slice());
    }
  }
}

#[cfg(test)]
mod v2_test {
  use crate::v2::encoder::ToBytes;
  use crate::v2::decoder::FromBytes;

  #[test]
  fn test_slice() {
    let source = [69u32; 10];
    println!("Source: {:?}", source);

    let bytes = source.to_bytes();
    let data = Vec::<u32>::from_bytes(bytes);
    println!("From: {:?}", data);

    if let Ok(data) = data {
      assert_eq!(source, data.as_slice());
    }
  }
}

