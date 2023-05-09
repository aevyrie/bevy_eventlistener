use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(EntityEvent, attributes(target))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let mut target = None;

    match ast.data {
        // Only process structs
        syn::Data::Struct(ref data_struct) => {
            // Check the kind of fields the struct contains
            match data_struct.fields {
                // Structs with named fields
                syn::Fields::Named(ref fields_named) => {
                    // Iterate over the fields
                    for field in fields_named.named.iter() {
                        // Get attributes #[..] on each field
                        for attr in field.attrs.iter() {
                            // Parse the attribute
                            match attr.meta {
                                // Find the duplicated idents
                                syn::Meta::Path(ref path)
                                    if path.get_ident().unwrap().to_string() == "target" =>
                                {
                                    target = Some(field.ident.clone());
                                }
                                _ => (),
                            }
                        }
                    }
                }

                // Struct with unnamed fields
                _ => (),
            }
        }

        // Panic when we don't have a struct
        _ => panic!("Must be a struct"),
    }

    let target = target.unwrap();

    let gen = quote! {
        impl EntityEvent for #name {
            fn target(&self) -> Entity {
                self.#target
            }
        }
    };
    gen.into()
}
