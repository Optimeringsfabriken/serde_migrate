// #![feature(trace_macros)]
// #![feature(log_syntax)]

use proc_macro::TokenStream;
use quote::{quote, ToTokens, format_ident};

use syn::{parse_macro_input, DeriveInput, Expr, ExprLit, punctuated::Punctuated, Ident, token::Comma, Visibility, parse_quote};

/// Macro for generating versioned serde serialization and deserialization implementations.
///
/// See the crate-level documentation for more information.
#[proc_macro_attribute]
pub fn versioned(_root_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let original_ast = parse_macro_input!(item as DeriveInput);

    let mut versioned_ast = original_ast.clone();

    let (impl_generics, generics, _) = original_ast.generics.split_for_impl();
    let mut generics_with_de_lifetime = original_ast.generics.clone();
    generics_with_de_lifetime.params.push(parse_quote!('de));

    let mut generics_with_a_lifetime = original_ast.generics.clone();
    generics_with_a_lifetime.params.push(parse_quote!('a));

    // ::<T>, or empty if there are no generics
    let turbo_generics = if generics.to_token_stream().is_empty() {
        quote!()
    } else {
        quote!(::#generics)
    };


    let struct_name = original_ast.ident.clone();

    let extra_ast;

    match &mut versioned_ast.data {
        syn::Data::Struct(ref mut struct_data) => {
            match &mut struct_data.fields {
                // for named field structs e.g. { A: int }
                syn::Fields::Named(fields) => {
                    let mut max_version = 1;
                    let mut versions = vec![];
                    for field in fields.named.iter() {
                        let mut start = 1;
                        let mut end = None;
                        for attr in &field.attrs {
                            if attr.path().is_ident("version") {
                                let expr = attr.parse_args_with(Punctuated::<Expr, Comma>::parse_terminated).expect("version attribute must be on the form `version(start=x, end=y)`");
                                for param in expr {
                                    match param {
                                        Expr::Assign(assign) => {
                                            let s = match *assign.left {
                                                Expr::Path(path) => {
                                                    if path.path.is_ident("start") {
                                                        "start"
                                                    } else if path.path.is_ident("end") {
                                                        "end"
                                                    } else {
                                                        return syn::Error::new_spanned(path.to_token_stream(), "Unknown attribute. Expected 'start' or 'end'".to_string()).to_compile_error().into();
                                                    }
                                                }
                                                _ => return syn::Error::new_spanned(assign.left.to_token_stream(), "Unknown attribute. Expected 'start' or 'end'".to_string()).to_compile_error().into(),
                                            };

                                            let v: u32 = match &*assign.right {
                                                Expr::Lit(ExprLit { lit: syn::Lit::Int(lit), .. }) => {
                                                    lit.base10_parse().expect("Invalid version number")
                                                }
                                                _ => {
                                                    return syn::Error::new_spanned(assign.right.to_token_stream(), "Expected positive integer".to_string()).to_compile_error().into()
                                                }
                                            };

                                            if v == 0 {
                                                return syn::Error::new_spanned(assign.right.to_token_stream(), "Version numbers start at 1".to_string()).to_compile_error().into()
                                            }

                                            max_version = max_version.max(v);
                                            if s == "end" && v == 1 {
                                                return syn::Error::new_spanned(assign.right.to_token_stream(), "Cannot remove fields in the first version".to_string()).to_compile_error().into()
                                            }

                                            if s == "start" {
                                                start = v;
                                            }
                                            if s == "end" {
                                                end = Some(v);
                                            }
                                        },
                                        _ => return syn::Error::new_spanned(param.to_token_stream(), "version attribute must be on the form `version(start=x, end=y)`".to_string()).to_compile_error().into(),
                                    }
                                }
                            }

                            if Some(start) == end {
                                return syn::Error::new_spanned(attr.to_token_stream(), "Cannot remove field in the same version it was added".to_string()).to_compile_error().into()
                            }
                        }
                        versions.push((start, end));
                    }

                    let min_version: u32 = 1;

                    if max_version == min_version {
                        // return syn::Error::new_spanned(original_ast, "No versions were specified. Specify versions using the #[version(min=int, max=int)] attribute on fields.".to_string()).to_compile_error().into()
                    }

                    for field in fields.named.iter_mut() {
                        field.attrs.retain(|a| !a.path().is_ident("version"));
                    }

                    let mut versioned_structs = quote!();
                    let mut version_struct_names = vec![];
                    let mut versioned_variants = quote!();
                    for v in min_version..=max_version {
                        let mut versioned_fields = quote!();
                        for (i, field) in fields.named.iter().enumerate() {
                            let (start, end) = versions[i];
                            if start <= v && end.unwrap_or(u32::MAX) > v {
                                let mut field = field.clone();
                                // Make the field public if it isn't already
                                if !matches!(field.vis, Visibility::Public(_)) {
                                    field.vis = Visibility::Public(Default::default());
                                }
                                versioned_fields.extend(quote!(
                                    #field,
                                ));
                            }
                        }

                        let versioned_name = format_ident!("{}V{}", struct_name, v.to_string());
                        let variant_name = format_ident!("V{}", v.to_string());
                        version_struct_names.push(versioned_name.clone());

                        versioned_structs.extend(quote!(
                            #[derive(serde::Deserialize)]
                            pub(crate) struct #versioned_name #generics {
                                #versioned_fields
                            }
                        ));

                        versioned_variants.extend(quote!(#variant_name(#versioned_name #generics),))
                    }

                    // Remove all fields that are removed in the latest version
                    fields.named = fields.named.clone().into_iter().enumerate().filter_map(|(i, f)| {
                        let (_, end) = versions[i];
                        if end.is_none() {
                            Some(f)
                        } else {
                            None
                        }
                    }).collect();

                    let mut migration_fns = quote!();
                    for v in (min_version+1)..=max_version {
                        let fn_name = format_ident!("to_v{}", v.to_string());
                        let from = &version_struct_names[((v-1) - min_version) as usize];
                        let to = &version_struct_names[(v - min_version) as usize];
                        migration_fns.extend(quote!(
                            fn #fn_name (v: #from #generics) -> #to #generics;
                        ));
                    }

                    let migration_trait = quote! {
                        pub(crate) trait Migrate #generics {
                            #migration_fns
                        }
                    };

                    let mod_name = format_ident!("{}_migrations", struct_name.to_string().to_lowercase());
                    let version_struct_idents = version_struct_names.iter().cloned().collect::<Punctuated<Ident,Comma>>();

                    // We need a version of the struct with all fields borrowed to be able to serialize it.
                    // In particular, we want to auto-derive the Serialize trait, and we don't want to assume
                    // all fields are copy or clone.
                    let borrowed_fields = fields.named.iter().map(|f| {
                        let mut f = f.clone();
                        f.vis = Visibility::Public(Default::default());
                        let ty = &f.ty;
                        f.ty = parse_quote!(&'a #ty);
                        quote!(#f)
                    }).collect::<Punctuated<_,Comma>>();

                    let mut migration_calls = quote!();
                    for v in min_version..=max_version {
                        let from_variant = format_ident!("V{}", v.to_string());
                        let to_variant = format_ident!("V{}", (v+1).to_string());
                        let fn_name = format_ident!("to_v{}", (v+1).to_string());
                        if v == max_version {
                            migration_calls.extend(quote!(
                                DataVersions #turbo_generics::#from_variant(data) => #struct_name #turbo_generics::from(data),
                            ));
                        } else {
                            migration_calls.extend(quote!(
                                DataVersions #turbo_generics::#from_variant(data) => DataVersions #turbo_generics::#to_variant(#struct_name #turbo_generics::#fn_name(data)).migrate(),
                            ));
                        }
                    }

                    // We need to have a concrete struct even for the last version because otherwise
                    // we cannot auto-derive the deserialization implementation for it.
                    // The only thing we will do with it is to convert it to the user-defined struct
                    let from_last_impl = {
                        let last_version = version_struct_names.last().unwrap();
                        let field_copies = fields.named.iter().map(|f| {
                            let name = f.ident.as_ref().unwrap();
                            quote!(#name : v.#name)
                        }).collect::<Punctuated<_,Comma>>();

                        quote!{
                            impl #impl_generics From<#last_version #generics> for #struct_name #generics {
                                fn from(v: #last_version #generics) -> Self {
                                    Self {
                                        #field_copies
                                    }
                                }
                            }
                        }
                    };

                    let field_copies_from_self = fields.named.iter().map(|f| {
                        let name = f.ident.as_ref().unwrap();
                        quote!(#name : &self.#name)
                    }).collect::<Punctuated<_,Comma>>();

                    let versioned_deserialization_cases = (min_version..=max_version).map(|v| {
                        let variant_name = format_ident!("V{}", v.to_string());
                        let versioned_struct_name = &version_struct_names[(v-min_version) as usize];
                        quote!(#v => DataVersions #turbo_generics::#variant_name(#mod_name::#versioned_struct_name::deserialize(deserializer)?))
                    }).collect::<Punctuated<_,Comma>>();
                    let invalid_version_message = format!("Invalid version for {} (got {{}})", struct_name);

                    extra_ast = quote! {
                        pub(crate) mod #mod_name {
                            use super::*;
                            use super::#struct_name;
                            use serde::Deserialize;

                            #versioned_structs

                            #migration_trait

                            pub(crate) mod serialization_helpers {
                                use super::*;
                                use super::{#struct_name, Migrate, #version_struct_idents};
                                use serde::Serialize;

                                pub(crate) enum DataVersions #generics {
                                    #versioned_variants
                                }

                                #[derive(Serialize)]
                                pub(crate) struct Borrowed #generics_with_a_lifetime {
                                    #borrowed_fields
                                }

                                impl #impl_generics DataVersions #generics {
                                    pub(crate) fn migrate(self: DataVersions #generics) -> #struct_name #generics {
                                        match self {
                                            #migration_calls
                                        }
                                    }
                                }

                                #from_last_impl
                            }
                        }

                        impl #impl_generics serde::ser::Serialize for #struct_name #generics {
                            fn serialize<S>(&self, mut serializer: S) -> Result<S::Ok, S::Error>
                            where
                                S: serde::ser::Serializer,
                            {
                                use #mod_name::serialization_helpers::Borrowed;
                                use serde::ser::SerializeStruct;

                                // Poor man's specialization
                                // In release mode, this will compile down to nothing, since type_name is a const function.
                                if std::any::type_name::<S>() == std::any::type_name::<&mut serde_migrate::VersionSerializer>() {
                                    unsafe {
                                        let state: &mut &mut serde_migrate::VersionSerializer = std::mem::transmute(&mut serializer);
                                        state.set_version::<Self>(#max_version);
                                    }
                                }

                                Borrowed #generics {
                                    #field_copies_from_self
                                }.serialize(serializer)
                            }
                        }

                        impl #generics_with_de_lifetime serde::de::Deserialize<'de> for #struct_name #generics {
                            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                            where
                                D: serde::de::Deserializer<'de>,
                            {
                                use #mod_name::serialization_helpers::DataVersions;

                                let v = serde_migrate::DESERIALIZATION_STATE.with(|state| {
                                    let mut state = state.borrow_mut();
                                    if let Some(state) = &mut *state {
                                        state.get_version::<Self, D>()
                                    } else {
                                        Ok(#max_version)
                                    }
                                })?;

                                let result = match v {
                                    #versioned_deserialization_cases,
                                    _ => return Err(serde::de::Error::custom(format!(#invalid_version_message, v))),
                                };
                                Ok(result.migrate())
                            }
                        }
                    };
                }
                // for unit types e.g. A()
                syn::Fields::Unit | syn::Fields::Unnamed(_) => {
                    unimplemented!()
                }
            }
        }
        _ => panic!("`versioned` has to be used with structs "),
    }

    (quote! {
        #versioned_ast

        #extra_ast
    }).into()
}

