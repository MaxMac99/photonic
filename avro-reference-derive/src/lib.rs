use darling::FromAttributes;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde_json::{from_str, Value};
use syn::{
    parse_macro_input, spanned::Spanned, AttrStyle, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, ExprPath, Fields, Meta, Type, TypePath,
};

#[derive(FromAttributes)]
#[darling(attributes(avro))]
struct FieldOptions {
    #[darling(default)]
    doc: Option<String>,
    #[darling(default)]
    default: Option<String>,
    #[darling(multiple)]
    alias: Vec<String>,
    #[darling(default)]
    rename: Option<String>,
    #[darling(default)]
    skip: bool,
    #[darling(default)]
    reference: bool,
    #[darling(default)]
    replace: Option<ExprPath>,
}

#[derive(FromAttributes)]
#[darling(attributes(avro))]
struct NamedTypeOptions {
    #[darling(default)]
    namespace: Option<String>,
    #[darling(default)]
    doc: Option<String>,
    #[darling(multiple)]
    alias: Vec<String>,
    #[darling(default)]
    referencable: bool,
}

#[proc_macro_derive(AvroReferenceSchema, attributes(avro))]
// Templated from Serde
pub fn proc_macro_derive_avro_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    derive_avro_schema(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn derive_avro_schema(input: &mut DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let named_type_options =
        NamedTypeOptions::from_attributes(&input.attrs[..]).map_err(darling_to_syn)?;
    let full_schema_name = vec![named_type_options.namespace, Some(input.ident.to_string())]
        .into_iter()
        .flatten()
        .collect::<Vec<String>>()
        .join(".");

    let (schema_def, references) = match &input.data {
        Data::Struct(s) => get_data_struct_schema_def(
            &full_schema_name,
            named_type_options
                .doc
                .or_else(|| extract_outer_doc(&input.attrs)),
            named_type_options.alias,
            s,
            input.ident.span(),
        )?,
        Data::Enum(e) => get_data_enum_schema_def(
            &full_schema_name,
            named_type_options
                .doc
                .or_else(|| extract_outer_doc(&input.attrs)),
            named_type_options.alias,
            e,
            input.ident.span(),
        )?,
        _ => {
            return Err(vec![syn::Error::new(
                input.ident.span(),
                "AvroSchema only supports structs and enums",
            )])
        }
    };
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let referencable = if named_type_options.referencable {
        quote! {
            impl #impl_generics avro_reference::AvroReferencable for #ident #ty_generics #where_clause {
                fn get_referenced_schema() -> avro_reference::ReferenceSchema {
                    let name = apache_avro::schema::Name::new(#full_schema_name)
                        .expect(&format!("Unable to parse schema name {}", #full_schema_name)[..]);
                    avro_reference::ReferenceSchema {
                        schema: apache_avro::schema::Schema::Ref{ name },
                        references: vec![#(#references),*],
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    Ok(quote! {
        #referencable
        impl #impl_generics avro_reference::AvroReferenceSchemaComponent for #ident #ty_generics #where_clause {
            fn get_schema_in_ctx(named_schemas: &mut std::collections::HashMap<apache_avro::schema::Name, apache_avro::schema::Schema>, enclosing_namespace: &Option<String>) -> avro_reference::ReferenceSchema {
                let name = apache_avro::schema::Name::new(#full_schema_name)
                    .expect(&format!("Unable to parse schema name {}", #full_schema_name)[..])
                    .fully_qualified_name(enclosing_namespace);
                let enclosing_namespace = &name.namespace;
                let schema = if named_schemas.contains_key(&name) {
                    apache_avro::schema::Schema::Ref{ name }
                } else {
                    named_schemas.insert(name.clone(), apache_avro::schema::Schema::Ref{ name: name.clone() });
                    #schema_def
                };
                avro_reference::ReferenceSchema {
                    schema,
                    references: vec![#(#references),*],
                }
            }
        }
    })
}

fn get_data_struct_schema_def(
    full_schema_name: &str,
    record_doc: Option<String>,
    aliases: Vec<String>,
    s: &DataStruct,
    error_span: Span,
) -> Result<(TokenStream, Vec<TokenStream>), Vec<syn::Error>> {
    let mut references = vec![];
    let mut record_field_exprs = vec![];
    match s.fields {
        Fields::Named(ref a) => {
            let mut index: usize = 0;
            for field in a.named.iter() {
                let mut name = field.ident.as_ref().unwrap().to_string();
                if let Some(raw_name) = name.strip_prefix("r#") {
                    name = raw_name.to_string();
                }
                let field_attrs =
                    FieldOptions::from_attributes(&field.attrs[..]).map_err(darling_to_syn)?;
                if field_attrs.skip {
                    continue;
                }

                if let Some(rename) = field_attrs.rename {
                    name = rename;
                }

                let doc =
                    preserve_optional(field_attrs.doc.or_else(|| extract_outer_doc(&field.attrs)));
                let default_value = match field_attrs.default {
                    None => quote! { None },
                    Some(default_value) => {
                        let _: Value = from_str(&default_value[..]).map_err(|e| {
                            vec![syn::Error::new(
                                field.ident.span(),
                                format!("Invalid avro default json: \n{e}"),
                            )]
                        })?;
                        quote! {
                            Some(serde_json::from_str(#default_value).expect(format!("Invalid JSON: {:?}", #default_value).as_str()))
                        }
                    }
                };

                let aliases = preserve_vec(field_attrs.alias);
                let position = index;
                let schema_expr = if field_attrs.reference {
                    references.push(field_attrs.replace.clone()
                        .map(|replacement| Ok(quote! {
                            <#replacement as avro_reference::AvroReferencable>::get_referenced_schema()
                        }))
                        .unwrap_or_else(|| type_to_referenced_expr(&field.ty, false))?);
                    field_attrs
                        .replace
                        .map(|replacement| Ok(quote! {
                            <#replacement as avro_reference::AvroReferencable>::get_referenced_schema().schema
                        }))
                        .unwrap_or_else(|| type_to_referenced_expr(&field.ty, true))?
                } else {
                    field_attrs
                        .replace
                        .clone()
                        .map(|replacement| Ok(quote! {
                        <#replacement as avro_reference::AvroReferenceSchemaComponent>::get_schema_in_ctx(named_schemas, enclosing_namespace).schema
                    }))
                        .unwrap_or_else(|| type_to_schema_expr(&field.ty))?
                };
                record_field_exprs.push(quote! {
                    apache_avro::schema::RecordField {
                        name: #name.to_string(),
                        doc: #doc,
                        default: #default_value,
                        aliases: #aliases,
                        schema: #schema_expr,
                        order: apache_avro::schema::RecordFieldOrder::Ascending,
                        position: #position,
                        custom_attributes: Default::default(),
                    }
                });
                index += 1;
            }
        }
        Fields::Unnamed(_) => {
            return Err(vec![syn::Error::new(
                error_span,
                "AvroSchema derive does not work for tuple structs",
            )])
        }
        Fields::Unit => {
            return Err(vec![syn::Error::new(
                error_span,
                "AvroSchema derive does not work for unit structs",
            )])
        }
    }

    let record_doc = preserve_optional(record_doc);
    let record_aliases = preserve_vec(aliases);
    references.dedup_by_key(|reference| reference.to_string());

    Ok((
        quote! {
            let schema_fields = vec![#(#record_field_exprs),*];
            let name = apache_avro::schema::Name::new(#full_schema_name)
                .expect(&format!("Unable to parse struct name for schema {}", #full_schema_name)[..]);
            let lookup: std::collections::BTreeMap<String, usize> = schema_fields
                .iter()
                .map(|field| (field.name.to_owned(), field.position))
                .collect();
            apache_avro::schema::Schema::Record(apache_avro::schema::RecordSchema {
                name,
                aliases: #record_aliases,
                doc: #record_doc,
                fields: schema_fields,
                lookup,
                attributes: Default::default(),
            })
        },
        references,
    ))
}

fn get_data_enum_schema_def(
    full_schema_name: &str,
    doc: Option<String>,
    aliases: Vec<String>,
    e: &DataEnum,
    error_span: Span,
) -> Result<(TokenStream, Vec<TokenStream>), Vec<syn::Error>> {
    let doc = preserve_optional(doc);
    let enum_aliases = preserve_vec(aliases);
    if e.variants.iter().all(|v| Fields::Unit == v.fields) {
        let default_value = default_enum_variant(e, error_span)?;
        let default = preserve_optional(default_value);
        let symbols: Vec<String> = e
            .variants
            .iter()
            .map(|variant| variant.ident.to_string())
            .collect();
        Ok((
            quote! {
                apache_avro::schema::Schema::Enum(apache_avro::schema::EnumSchema {
                    name: apache_avro::schema::Name::new(#full_schema_name).expect(&format!("Unable to parse enum name for schema {}", #full_schema_name)[..]),
                    aliases: #enum_aliases,
                    doc: #doc,
                    symbols: vec![#(#symbols.to_owned()),*],
                    default: #default,
                    attributes: Default::default(),
                })
            },
            vec![],
        ))
    } else {
        Err(vec![syn::Error::new(
            error_span,
            "AvroSchema derive does not work for enums with non unit structs",
        )])
    }
}

fn type_to_referenced_expr(ty: &Type, nested: bool) -> Result<TokenStream, Vec<syn::Error>> {
    match ty {
        Type::Path(p) => {
            let type_string = p.path.segments.last().unwrap().ident.to_string();
            let schema = match &type_string[..] {
                "bool" | "i8" | "i16" | "i32" | "u8" | "u16" | "u32" | "i64" | "f32" | "f64"
                | "String" | "str" | "char" | "u64" => {
                    return Err(vec![syn::Error::new_spanned(
                        ty,
                        "Default types cannot be referenced",
                    )])
                }
                _ => {
                    if nested {
                        quote! {<#p as avro_reference::AvroReferencable>::get_referenced_schema().schema }
                    } else {
                        quote! {<#p as avro_reference::AvroReferencable>::get_referenced_schema() }
                    }
                }
            };
            Ok(schema)
        }
        Type::Array(ta) => {
            let inner_schema_expr = crate::type_to_referenced_expr(&ta.elem, nested)?;
            Ok(if nested {
                quote! { apache_avro::schema::Schema::array(#inner_schema_expr) }
            } else {
                quote! { #inner_schema_expr }
            })
        }
        Type::Reference(tr) => crate::type_to_referenced_expr(&tr.elem, nested),
        _ => Err(vec![syn::Error::new_spanned(
            ty,
            format!("Unable to generate schema for type {ty:?}"),
        )]),
    }
}

fn type_to_schema_expr(ty: &Type) -> Result<TokenStream, Vec<syn::Error>> {
    match ty {
        Type::Path(p) => {
            let type_string = p.path.segments.last().unwrap().ident.to_string();
            let schema = match &type_string[..] {
                "bool" => quote! { apache_avro::schema::Schema::Boolean },
                "i8" | "i16" | "i32" | "u8" | "u16" => quote! { apache_avro::schema::Schema::Int },
                "u32" | "i64" => quote! { apache_avro::schema::Schema::Long },
                "f32" => quote! { apache_avro::schema::Schema::Float },
                "f64" => quote! { apache_avro::schema::Schema::Double },
                "String" | "str" => quote! { apache_avro::schema::Schema::String },
                "char" => {
                    return Err(vec![syn::Error::new_spanned(
                        ty,
                        "AvroSchema: Cannot guarantee successful deserialization of this type",
                    )])
                }
                "u64" => {
                    return Err(vec![syn::Error::new_spanned(
                        ty,
                        "Cannot guarantee successful serialization of this type due to overflow concerns",
                    )])
                } // Can't guarantee serialization type
                _ => {
                    type_path_schema_expr(p)
                }
            };
            Ok(schema)
        }
        Type::Array(ta) => {
            let inner_schema_expr = type_to_schema_expr(&ta.elem)?;
            Ok(quote! { apache_avro::schema::Schema::array(#inner_schema_expr) })
        }
        Type::Reference(tr) => type_to_schema_expr(&tr.elem),
        _ => Err(vec![syn::Error::new_spanned(
            ty,
            format!("Unable to generate schema for type {ty:?}"),
        )]),
    }
}

/// Generates the schema def expression for fully qualified type paths using the associated function
/// - `A -> <A as avro_reference::AvroReferenceSchemaComponent>::get_schema_in_ctx()`
/// - `A<T> -> <A<T> as avro_reference::AvroReferenceSchemaComponent>::get_schema_in_ctx()`
fn type_path_schema_expr(p: &TypePath) -> TokenStream {
    quote! {<#p as avro_reference::AvroReferenceSchemaComponent>::get_schema_in_ctx(named_schemas, enclosing_namespace).schema}
}

fn default_enum_variant(
    data_enum: &DataEnum,
    error_span: Span,
) -> Result<Option<String>, Vec<syn::Error>> {
    match data_enum
        .variants
        .iter()
        .filter(|v| v.attrs.iter().any(is_default_attr))
        .collect::<Vec<_>>()
    {
        variants if variants.is_empty() => Ok(None),
        single if single.len() == 1 => Ok(Some(single[0].ident.to_string())),
        multiple => Err(vec![syn::Error::new(
            error_span,
            format!(
                "Multiple defaults defined: {:?}",
                multiple
                    .iter()
                    .map(|v| v.ident.to_string())
                    .collect::<Vec<String>>()
            ),
        )]),
    }
}

fn is_default_attr(attr: &Attribute) -> bool {
    matches!(attr, Attribute { meta: Meta::Path(path), .. } if path.get_ident().map(Ident::to_string).as_deref() == Some("default"))
}

fn extract_outer_doc(attributes: &[Attribute]) -> Option<String> {
    let doc = attributes
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer && attr.path().is_ident("doc"))
        .filter_map(|attr| {
            let name_value = attr.meta.require_name_value();
            match name_value {
                Ok(name_value) => match &name_value.value {
                    syn::Expr::Lit(expr_lit) => match expr_lit.lit {
                        syn::Lit::Str(ref lit_str) => Some(lit_str.value().trim().to_string()),
                        _ => None,
                    },
                    _ => None,
                },
                Err(_) => None,
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    if doc.is_empty() {
        None
    } else {
        Some(doc)
    }
}

fn preserve_optional(op: Option<impl quote::ToTokens>) -> TokenStream {
    match op {
        Some(tt) => quote! {Some(#tt.into())},
        None => quote! {None},
    }
}

fn preserve_vec(op: Vec<impl quote::ToTokens>) -> TokenStream {
    let items: Vec<TokenStream> = op.iter().map(|tt| quote! {#tt.into()}).collect();
    if items.is_empty() {
        quote! {None}
    } else {
        quote! {Some(vec![#(#items),*])}
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let test = quote! {
            struct A {
                a: i32,
            }
        };

        match syn::parse2::<DeriveInput>(test) {
            Ok(mut input) => {
                assert!(derive_avro_schema(&mut input).is_ok())
            }
            Err(error) => panic!("Failed to parse as derive input. Error: {error:?}"),
        }
    }
}
