use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::derive2::{ctx::Context, derive, r#impl::Impl};

use super::{r#struct::Struct, variant::Variant, DeriveKind};

macro_rules! map_fields {
    ($struct:ident, $method:ident) => {
        $struct.fields().iter().map(|field| field.$method())
    };

    ($struct:ident, $method:ident ($($rest:tt)*)) => {
        $struct.fields().iter().map(|field| field.$method($($rest)*))
    };
}

pub struct Enum<'a> {
    derive: &'a DeriveKind,
    context: &'a Context,
    r#impl: &'a Impl<'a>,
    variants: Vec<Struct<'a>>,
}

impl<'a> Enum<'a> {
    pub fn parse(
        derive: &'a DeriveKind,
        context: &'a Context,
        r#impl: &'a Impl<'_>,
        data: &'a mut syn::DataEnum,
    ) -> derive::Result<Self> {
        if data.variants.is_empty() {
            context.error(&data.variants, "cannot derive for enums with zero variants");
            return Err(());
        }

        Self::parse_variants(derive, context, r#impl, &mut data.variants).map(|variants| Self {
            derive,
            context,
            r#impl,
            variants,
        })
    }

    fn parse_variants(
        derive: &'a DeriveKind,
        context: &'a Context,
        r#impl: &'a Impl<'_>,
        variants: &'a mut syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    ) -> derive::Result<Vec<Struct<'a>>> {
        let len = variants.iter().len();
        let mut parsed_data = Vec::with_capacity(len);

        for variant in variants.iter() {
            if variant.discriminant.is_some() {
                context.error(&variants, "cannot derive for enums with discriminants");
                return Err(());
            }
        }

        for variant in variants.iter_mut() {
            if let Ok((parsed, unknown_attrs)) = Variant::parse(context, variant) {
                parsed_data.push((parsed, unknown_attrs, &mut variant.fields));
            }
        }

        if parsed_data.len() != len {
            return Err(());
        }

        let mut tags = HashSet::new();
        let mut unique = true;

        for (variant, _, _) in &parsed_data {
            let (tag, tokens) = variant.tag_with_tokens();

            if !tags.insert(tag) {
                context.error(tokens, "duplicate tag");
                unique = false;
            }
        }

        let mut parsed = Vec::with_capacity(len);

        for (variant, unknown_attrs, fields) in parsed_data {
            if let Ok(r#struct) =
                Struct::parse(derive, context, r#impl, unknown_attrs, fields, variant)
            {
                parsed.push(r#struct);
            }
        }

        if parsed.len() != len {
            return Err(());
        }

        if unique {
            Ok(parsed)
        } else {
            Err(())
        }
    }

    fn impl_default(&self) -> TokenStream {
        let default = self.variants.iter().find_map(|r#struct| {
            let variant = r#struct
                .variant()
                .unwrap_or_else(|| unreachable!("expected a variant"));

            if variant.tag() == 0 {
                Some(r#struct.default())
            } else {
                None
            }
        });

        if let Some(default) = default {
            self.r#impl.impl_for(
                "Default",
                quote! {
                    fn default() -> Self {
                        #default
                    }
                },
            )
        } else {
            if self.derive == &DeriveKind::Deserialize || self.derive == &DeriveKind::State {
                self.context.error(
                    self.r#impl.name(),
                    "expected a variant with tag 0 as the default variant of this enum",
                );
            }

            TokenStream::new()
        }
    }

    fn impl_wire_type(&self) -> TokenStream {
        self.r#impl.impl_for(
            "WireType",
            quote! {
                const WIRE_TYPE: u8 = 2;
            },
        )
    }

    fn impl_serialize(&self) -> TokenStream {
        let name = self.r#impl.name();

        let sizers = self.variants.iter().map(|r#struct| {
            let variant = r#struct
                .variant()
                .unwrap_or_else(|| unreachable!("expected a variant"));

            let qual = variant.qual();
            let tag = variant.tag();

            let destructure = map_fields!(r#struct, destructure);
            let sizers = map_fields!(r#struct, sizer(true));

            quote! {
                #name #qual { #(#destructure,)* .. } => {
                    size += #tag.size();
                    #(#sizers)*
                }
            }
        });

        let serializers = self.variants.iter().map(|r#struct| {
            let variant = r#struct
                .variant()
                .unwrap_or_else(|| unreachable!("expected a variant"));

            let qual = variant.qual();
            let tag = variant.tag();

            let destructure = map_fields!(r#struct, destructure);
            let serializers = map_fields!(r#struct, serializer(true));

            quote! {
                #name #qual { #(#destructure,)* .. } => {
                    #tag.serialize(writer)?;
                    #(#serializers)*
                }
            }
        });

        self.r#impl.impl_for(
            "Serialize2",
            quote! {
                fn size(&self) -> u32 {
                    let mut size = 0;
                    match self { #(#sizers)* }
                    size
                }

                fn serialize(&self, writer: &mut impl io::Write) -> io::Result<()> {
                    match self { #(#serializers)* }
                    Ok(())
                }
            },
        )
    }
}

impl<'a> ToTokens for Enum<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.impl_default());
        tokens.extend(self.impl_wire_type());
        tokens.extend(self.impl_serialize());
    }
}