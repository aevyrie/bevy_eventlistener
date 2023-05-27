use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(EntityEvent, attributes(target))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident;

    let mut target = None;

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
                        match attr.meta {
                            // Find the duplicated idents
                            syn::Meta::Path(ref path) if path.get_ident().unwrap() == "target" => {
                                target = Some(field.ident.clone());
                            }
                            _ => (),
                        }
                    }
                }
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
