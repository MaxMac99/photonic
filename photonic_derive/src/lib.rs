use darling::FromAttributes;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DataStruct, DeriveInput, Fields};

#[derive(FromAttributes)]
#[darling(attributes(event))]
struct FieldOptions {
    #[darling(default)]
    key: bool,
}

#[derive(FromAttributes)]
#[darling(attributes(event))]
struct NamedTypeOptions {
    #[darling(default)]
    topic: String,
}

#[proc_macro_derive(Event, attributes(event))]
pub fn proc_macro_derive_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    derive_event(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn derive_event(input: &mut DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let named_type_options =
        NamedTypeOptions::from_attributes(&input.attrs[..]).map_err(darling_to_syn)?;
    let ident = &input.ident;

    let keyed_impl = match &input.data {
        Data::Struct(s) => create_keyed_impl(ident, s)?,
        _ => {
            return Err(vec![syn::Error::new(
                input.ident.span(),
                "Events only supports structs",
            )]);
        }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let topic = named_type_options.topic;
    Ok(quote! {
        impl #impl_generics crate::util::events::Event for #ident #ty_generics #where_clause {
            fn topic() -> &'static str {
                #topic
            }

            fn to_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }

            fn clone_box(&self) -> Box<dyn crate::util::events::Event + Send> {
                Box::new(self.clone())
            }
        }
        #keyed_impl
    })
}

fn create_keyed_impl(ident: &Ident, s: &DataStruct) -> Result<TokenStream, Vec<syn::Error>> {
    match s.fields {
        Fields::Named(ref a) => {
            for field in a.named.iter() {
                let field_attrs =
                    FieldOptions::from_attributes(&field.attrs[..]).map_err(darling_to_syn)?;
                let name = &field.ident;
                let type_name = &field.ty;
                if field_attrs.key {
                    return Ok(quote! {
                        impl crate::util::stream::Keyed<#type_name> for #ident {
                            fn get_key(&self) -> #type_name {
                                self.#name
                            }
                        }
                    });
                }
            }
        }
        _ => {}
    };
    Ok(quote! {})
}

fn to_compile_errors(errors: Vec<syn::Error>) -> TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}

fn darling_to_syn(e: darling::Error) -> Vec<syn::Error> {
    let msg = format!("{e}");
    let token_errors = e.write_errors();
    vec![syn::Error::new(token_errors.span(), msg)]
}
