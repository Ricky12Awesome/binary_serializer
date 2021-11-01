use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Ident, Path, Index, DeriveInput, FieldsNamed, FieldsUnnamed};

mod serialize {
  use crate::*;

  macro_rules! quote_serializer {
    ($id:ident: $($tt:tt)*) => {
      quote! {
        impl ::binary_serializer::encoder::Serializer for #$id {
          fn encode(&self, encoder: &mut impl ::binary_serializer::encoder::Encoder) {
            $($tt)*
          }
        }
      }
    };
  }

  pub(crate) fn struct_named(ident: Ident, fields: FieldsNamed) -> proc_macro2::TokenStream {
    let fields = fields.named.iter()
      .map(|f| &f.ident)
      .map(|name| quote! { encoder.encode_value(&self.#name) });

    quote_serializer! {
      ident: #(#fields);*
    }
  }

  pub(crate) fn struct_unnamed(ident: Ident, fields: FieldsUnnamed) -> proc_macro2::TokenStream {
    let fields = fields.unnamed.iter()
      .enumerate()
      .map(|(idx, _)| Index::from(idx))
      .map(|idx| quote! { encoder.encode_value(&self.#idx) });

    quote_serializer! {
      ident: #(#fields);*
    }
  }

  pub(crate) fn struct_unit(ident: Ident) -> proc_macro2::TokenStream {
    quote_serializer! {
      ident:
    }
  }
}

mod deserialize {
  use crate::*;

  macro_rules! quote_deserializer {
    ($id:ident: $($tt:tt)*) => {
      quote! {
        impl ::binary_serializer::decoder::Deserializer for #$id {
          fn decode(decoder: &mut impl ::binary_serializer::decoder::Decoder) -> ::binary_serializer::decoder::DecoderResult<Self> {
            Ok($($tt)*)
          }
        }
      }
    };
  }

  pub(crate) fn struct_named(ident: Ident, fields: FieldsNamed) -> proc_macro2::TokenStream {
    let fields = fields.named.iter()
      .map(|f| &f.ident)
      .map(|name| quote! { #name: decoder.decode_value()? });

    quote_deserializer! {
      ident: Self {
        #(#fields),*
      }
    }
  }

  pub(crate) fn struct_unnamed(ident: Ident, fields: FieldsUnnamed) -> proc_macro2::TokenStream {
    let fields = fields.unnamed.iter()
      .map(|_| quote! { decoder.decode_value()? });

    quote_deserializer! {
      ident: Self(#(#fields),*)
    }
  }

  pub(crate) fn struct_unit(ident: Ident) -> proc_macro2::TokenStream {
    quote_deserializer! {
      ident: Self
    }
  }
}


fn unimpl(_typ: &str) -> proc_macro2::TokenStream {
  quote! {
    compile_error!("Unimplemented: {}", _typ);
  }
}

#[proc_macro_derive(Serializer)]
pub fn serialize(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  let output = match data {
    syn::Data::Struct(s) => match s.fields {
      syn::Fields::Named(fields) => serialize::struct_named(ident, fields),
      syn::Fields::Unnamed(fields) => serialize::struct_unnamed(ident, fields),
      syn::Fields::Unit => serialize::struct_unit(ident),
    },
    syn::Data::Enum(_) => unimpl("Enums"),
    syn::Data::Union(_) => {
      unimpl("Union?")
    }
  };

  output.into()
}

#[proc_macro_derive(Deserializer)]
pub fn deserialize(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  let output = match data {
    syn::Data::Struct(s) => match s.fields {
      syn::Fields::Named(fields) => deserialize::struct_named(ident, fields),
      syn::Fields::Unnamed(fields) => deserialize::struct_unnamed(ident, fields),
      syn::Fields::Unit => deserialize::struct_unit(ident),
    },
    syn::Data::Enum(_) => unimpl("Enum"),
    syn::Data::Union(_) => {
      unimpl("Union?")
    }
  };

  output.into()
}