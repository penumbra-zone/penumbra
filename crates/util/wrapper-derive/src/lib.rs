extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(StateRead)]
pub fn state_read_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        use std::{any::Any, ops::RangeBounds};

        impl #impl_generics StateRead for #name #ty_generics #where_clause {
            type GetRawFut = S::GetRawFut;
            type PrefixRawStream = S::PrefixRawStream;
            type PrefixKeysStream = S::PrefixKeysStream;
            type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;
            type NonconsensusRangeRawStream = S::NonconsensusRangeRawStream;

            fn get_raw(&self, key: &str) -> Self::GetRawFut {
                self.0.get_raw(key)
            }

            fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
                self.0.nonverifiable_get_raw(key)
            }

            fn object_get<T: Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
                self.0.object_get(key)
            }

            fn object_type(&self, key: &'static str) -> Option<std::any::TypeId> {
                self.0.object_type(key)
            }

            fn prefix_raw(&self, prefix: &str) -> Self::PrefixRawStream {
                self.0.prefix_raw(prefix)
            }

            fn prefix_keys(&self, prefix: &str) -> Self::PrefixKeysStream {
                self.0.prefix_keys(prefix)
            }

            fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> Self::NonconsensusPrefixRawStream {
                self.0.nonverifiable_prefix_raw(prefix)
            }

            fn nonverifiable_range_raw(
                &self,
                prefix: Option<&[u8]>,
                range: impl RangeBounds<Vec<u8>>,
            ) -> Result<Self::NonconsensusRangeRawStream> {
                self.0.nonverifiable_range_raw(prefix, range)
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(StateWrite)]
pub fn state_write_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics StateWrite for #name #ty_generics #where_clause {
            fn put_raw(&mut self, key: String, value: Vec<u8>) {
                self.0.put_raw(key, value)
            }

            fn delete(&mut self, key: String) {
                self.0.delete(key)
            }

            fn nonverifiable_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
                self.0.nonverifiable_put_raw(key, value)
            }

            fn nonverifiable_delete(&mut self, key: Vec<u8>) {
                self.0.nonverifiable_delete(key)
            }

            fn object_put<T: Clone + std::any::Any + Send + Sync>(&mut self, key: &'static str, value: T) {
                self.0.object_put(key, value)
            }

            fn object_delete(&mut self, key: &'static str) {
                self.0.object_delete(key)
            }

            fn object_merge(&mut self, objects: std::collections::BTreeMap<&'static str, Option<Box<dyn std::any::Any + Send + Sync>>>) {
                self.0.object_merge(objects)
            }

            fn record(&mut self, event: tendermint::abci::Event) {
                self.0.record(event)
            }
        }
    };

    TokenStream::from(expanded)
}
