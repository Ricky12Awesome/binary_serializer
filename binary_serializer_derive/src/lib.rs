use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Ident, DataUnion, DeriveInput, FieldsNamed, Field};

mod serialize {
  use crate::*;

  pub(crate) fn struct_named(ident: Ident, fields: FieldsNamed) -> proc_macro2::TokenStream {
    let fields = fields.named.iter()
      .map(|f| &f.ident)
      .map(|name| quote! { encoder.encode_value(&self.#name) });

    quote! {
      impl Serializer for #ident {
        fn encode(&self, encoder: &mut impl Encoder) {
          #(#fields);*
        }
      }
    }
  }
}

mod deserialize {
  use crate::*;

  pub(crate) fn struct_named(ident: Ident, fields: FieldsNamed) -> proc_macro2::TokenStream {
    let fields = fields.named.iter()
      .map(|f| &f.ident)
      .map(|name| quote! { #name: decoder.decode_value()? });

    quote! {
      impl Deserializer for #ident {
        fn decode(decoder: &mut impl Decoder) -> DecoderResult<Self>  {
          Ok(Self {
            #(#fields),*
          })
        }
      }
    }
  }
}


fn unimpl() -> proc_macro2::TokenStream {
  quote! {}
}

#[proc_macro_derive(Serializer)]
pub fn serialize(input: TokenStream) -> TokenStream {
  let DeriveInput { ident, data, .. } = parse_macro_input!(input);

  let output = match data {
    syn::Data::Struct(s) => match s.fields {
      syn::Fields::Named(fields) => serialize::struct_named(ident, fields),

      syn::Fields::Unnamed(_) => unimpl(),
      syn::Fields::Unit => unimpl(),
    },
    syn::Data::Enum(_) => unimpl(),
    syn::Data::Union(
      DataUnion {
        fields: FieldsNamed { named, .. },
        ..
      }) => {
      unimpl()
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

      syn::Fields::Unnamed(_) => unimpl(),
      syn::Fields::Unit => unimpl(),
    },
    syn::Data::Enum(_) => unimpl(),
    syn::Data::Union(
      DataUnion {
        fields: FieldsNamed { named, .. },
        ..
      }) => {
      unimpl()
    }
  };

  output.into()
}