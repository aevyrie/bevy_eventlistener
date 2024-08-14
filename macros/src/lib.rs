use core::panic;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(EntityEvent, attributes(target, can_bubble))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut target = None;
    let mut can_bubble = false;

    for attr in ast.attrs.iter() {
        if attr.path().is_ident("can_bubble") {
            can_bubble = true;
        }
    }

    match ast.data {
        // Only process structs
        syn::Data::Struct(ref data_struct) => {
            // Check the kind of fields the struct contains
            // Structs with named fields
            if let syn::Fields::Named(ref fields_named) = data_struct.fields {
                // Iterate over the fields
                for field in fields_named.named.iter() {
                    // Get attributes #[..] on each field
                    for attr in field.attrs.iter() {
                        // Parse the attribute
                        if let syn::Meta::Path(ref path) = attr.meta {
                            if let Some(ident) = path.get_ident() {
                                if ident == "target" {
                                    target = Some(field.ident.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        // Panic when we don't have a struct
        _ => panic!("Must be a struct"),
    }

    let Some(target) = target else {
        panic!("Missing `#[target] attribute. You must annotate the field with the target Entity, or instead manually implement EntityEvent.")
    };

    let gen = quote! {
        impl #impl_generics EntityEvent for #name #ty_generics #where_clause {
            fn target(&self) -> Entity {
                self.#target
            }
            fn can_bubble(&self) -> bool {
                #can_bubble
            }
        }
    };
    gen.into()
}
