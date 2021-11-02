use proc_macro::{self, TokenStream};
use quote::{quote, format_ident};
use syn::{parse_macro_input, Ident, Fields, DataEnum, Index, DeriveInput, FieldsNamed, FieldsUnnamed};

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

  pub(crate) fn enum_(ident: Ident, data: DataEnum) -> proc_macro2::TokenStream {
    let enum_index = data.variants.iter()
      .enumerate()
      .map(|(idx, v)| {
        let name = &v.ident;
        let index = Index::from(idx);
        let stmt = match &v.fields {
          Fields::Named(_fields) => quote! { Self::#name { .. } => #index },
          Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter().map(|_| format_ident!("_"));

            quote! {
              Self::#name(#(#fields),*) => #index
            }
          }
          Fields::Unit => quote! { Self::#name => #index }
        };

        stmt
      });

    let enum_variants = data.variants.iter()
      .map(|v| {
        let name = &v.ident;
        let match_stmt = match &v.fields {
          Fields::Named(fields) => {
            let fields = fields.named.iter()
              .map(|f| &f.ident)
              .collect::<Vec<_>>();

            quote! {
              Self::#name { #(#fields),* } => {
                #(encoder.encode_value(#fields);)*
              }
            }
          }
          Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter()
              .enumerate()
              .map(|(idx, _)| format_ident!("_{}", Index::from(idx)))
              .collect::<Vec<_>>();

            quote! {
              Self::#name(#(#fields),*) => {
                #(encoder.encode_value(#fields);)*
              }
            }
          }
          Fields::Unit => {
            quote! {
              Self::#name => {}
            }
          }
        };

        match_stmt
      });

    quote_serializer! {
      ident:
      let index: usize = match self {
        #(#enum_index),*
      };

      encoder.encode_value(&index);

      match self {
        #(#enum_variants),*
      }
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
            $($tt)*
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
      ident: Ok(Self {
        #(#fields),*
      })
    }
  }

  pub(crate) fn struct_unnamed(ident: Ident, fields: FieldsUnnamed) -> proc_macro2::TokenStream {
    let fields = fields.unnamed.iter()
      .map(|_| quote! { decoder.decode_value()? });

    quote_deserializer! {
      ident: Ok(Self(#(#fields),*))
    }
  }

  pub(crate) fn struct_unit(ident: Ident) -> proc_macro2::TokenStream {
    quote_deserializer! {
      ident: Ok(Self)
    }
  }

  pub(crate) fn enum_(ident: Ident, data: DataEnum) -> proc_macro2::TokenStream {
    let enum_variants = data.variants.iter()
      .enumerate()
      .map(|(idx, v)| {
        let name = &v.ident;
        let index = Index::from(idx);
        let match_stmt = match &v.fields {
          Fields::Named(fields) => {
            let fields = fields.named.iter()
              .map(|f| &f.ident)
              .collect::<Vec<_>>();

            quote! {
              #index => Self::#name {
                #(#fields: decoder.decode_value()?),*
              }
            }
          }
          Fields::Unnamed(fields) => {
            let fields = fields.unnamed.iter()
              .map(|_| quote! { decoder.decode_value()? });

            quote! {
              #index => Self::#name(
                #(#fields),*
              )
            }
          }
          Fields::Unit => {
            quote! {
              #index => Self::#name
            }
          }
        };

        match_stmt
      });


    quote_deserializer! {
      ident:
      let index: usize = decoder.decode_value()?;

      Ok(match index {
        #(#enum_variants,)*
        _ => return Err(::binary_serializer::decoder::DecoderError::custom("Invalid Enum"))
      })
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
    syn::Data::Enum(data) => serialize::enum_(ident, data),
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
    syn::Data::Enum(data) => deserialize::enum_(ident, data),
    syn::Data::Union(_) => {
      unimpl("Union?")
    }
  };

  output.into()
}