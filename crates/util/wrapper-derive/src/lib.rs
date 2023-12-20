extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

fn is_arc_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        return type_path.path.segments.iter().any(|seg| seg.ident == "Arc");
    }

    if let syn::Type::Reference(type_ref) = ty {
        return is_arc_type(&type_ref.elem);
    }

    false
}

#[proc_macro_derive(StateRead)]
pub fn state_read_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let _ = match &input.data {
        syn::Data::Struct(s) => {
            &s.fields
                .iter()
                .next()
                .expect("struct must have one field")
                .ty
        }
        _ => panic!("StateRead can only be derived for structs"),
    };

    let expanded = quote! {
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

                fn object_get<T: std::any::Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
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
                    range: impl std::ops::RangeBounds<Vec<u8>>,
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

    let field_type = match &input.data {
        syn::Data::Struct(s) => {
            &s.fields
                .iter()
                .next()
                .expect("struct must have one field")
                .ty
        }
        _ => panic!("StateWrite can only be derived for structs"),
    };

    let expanded = if is_arc_type(field_type) {
        quote! {
            impl #impl_generics StateWrite for #name #ty_generics #where_clause {
                fn put_raw(&mut self, key: String, value: Vec<u8>) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.put_raw(key, value)
                }

                fn delete(&mut self, key: String) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.delete(key)
                }

                fn nonverifiable_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.nonverifiable_put_raw(key, value)
                }

                fn nonverifiable_delete(&mut self, key: Vec<u8>) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.nonverifiable_delete(key)
                }

                fn object_put<T: Clone + std::any::Any + Send + Sync>(&mut self, key: &'static str, value: T) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.object_put(key, value)
                }

                fn object_delete(&mut self, key: &'static str) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.object_delete(key)
                }

                fn object_merge(&mut self, objects: std::collections::BTreeMap<&'static str, Option<Box<dyn std::any::Any + Send + Sync>>>) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.object_merge(objects)
                }

                fn record(&mut self, event: tendermint::abci::Event) {
                    let state = std::sync::Arc::get_mut(&mut self.0).expect("state is not unique");
                    state.record(event)
                }
            }
        }
    } else {
        quote! {
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
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(ChainStateReadExt)]
pub fn chain_state_read_ext(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let _ = match &input.data {
        syn::Data::Struct(s) => {
            &s.fields
                .iter()
                .next()
                .expect("struct must have one field")
                .ty
        }
        _ => panic!("ChainStateReadExt can only be derived for structs"),
    };

    let expanded = quote! {
            #[async_trait::async_trait]
            impl #impl_generics ChainStateReadExt for #name #ty_generics #where_clause {
                async fn get_chain_id(&self) -> Result<String> {
                    self.0.get_chain_id().await
                }

                async fn get_revision_number(&self) -> Result<u64> {
                    self.0.get_revision_number().await
                }

                async fn get_block_height(&self) -> Result<u64> {
                    self.0.get_block_height().await
                }

                async fn get_block_timestamp(&self) -> Result<tendermint::Time> {
                    self.0.get_block_timestamp().await
                }
            }
    };

    TokenStream::from(expanded)
}
