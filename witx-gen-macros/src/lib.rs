//! This is highly experimental and entirely untested. Do not try to use it.

#![forbid(rust_2018_idioms, future_incompatible, elided_lifetimes_in_paths)]
#![warn(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

extern crate proc_macro;

mod array;
mod builtin;
mod builtin_type_ext;
mod const_pointer;
mod enum_type;
mod flags;
mod handle;
mod int;
mod int_repr_ext;
mod pointer;
mod string_ext;
mod struct_type;
mod to_ident;
mod token_stream_pair;
mod type_definition_extensions;
mod type_ref_ext;
mod union;

use self::{
    string_ext::StringExt, to_ident::ToIdent, token_stream_pair::TokenStreamPair,
    type_definition_extensions::TypeDefinitionExtensions, type_ref_ext::TypeRefExt,
};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use std::{env, path::Path};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::FatArrow,
    Index, LitStr,
};
use witx::Id;

struct Input {
    version: LitStr,
    _arrow: FatArrow,
    path: LitStr,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let version = Parse::parse(input)?;
        let arrow = Parse::parse(input)?;
        let path = Parse::parse(input)?;

        Ok(Input {
            version,
            _arrow: arrow,
            path,
        })
    }
}

/// Generate definitions from a witx file.
///
/// TODO: More details.
#[proc_macro]
pub fn witx_gen(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Input);

    let base_path = env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable is not set.");
    let path = Path::new(&base_path).join(&input.path.value());
    let version = input.version.value();

    let document = witx::load(std::slice::from_ref(&path)).expect("Unable to load witx document");

    let mut output = document
        .typenames()
        .map(|tp| tp.tref.get_type_definitions(&tp.name, tp.docs.as_docs()))
        .fold(TokenStreamPair::new(), |output, defs| output.extend(defs));

    // TODO: Clean this code up and move it out of this function and into a separate module.
    //       There is lots of duplication here.
    let wasi_snapshot_id = Id::new(&version);

    let wasi_module = document
        .module(&wasi_snapshot_id)
        .expect("Could not find the WASI snapshot module.");

    let wasi_trait_functions = wasi_module.funcs().map(|func| {
        let docs = func.docs.as_docs();
        let param_docs = func.params.iter().map(|p| {
            format!("* {}: {}", p.name.as_str(), p.docs).as_docs()
        });
        let result_docs = func.results.iter().map(|p| {
            format!("* {}", p.docs).as_docs()
        });
        let ident = func.name.to_ident_native(None);

        let params = func.params.iter().map(|p| {
            let ident = p.name.to_ident_native(None);

            if p.tref.is_string() {
                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                return quote! { #ident: witx_gen::WasmSlicePtr<u8>, #len_ident: size };
            }

            if let Some(inner) = p.tref.as_array() {
                let inner_type = inner.name.to_ident_native(None);
                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                return quote! { #ident: witx_gen::WasmSlicePtr<#inner_type>, #len_ident: size };
            }

            let tp =  p.tref.to_type();
            quote! { #ident: #tp }
        });

        let results = func.results.iter().map(|p| {
            let typename = p.tref.to_type();

            quote! { #typename }
        });

        quote! {
            #docs
            #[doc = "# Parameters"]
            #[doc = "* ctx: The WASM runtime context"]
            #( #param_docs )*
            #[doc = "# Results"]
            #( #result_docs )*
            fn #ident(&self, ctx: &mut witx_gen::reexports::Ctx, #( #params ),*) -> ( #( #results ),* );
        }
    });

    let wasi_trait_impls_arc = wasi_module.funcs().map(|func| {
        let ident = func.name.to_ident_native(None);

        let params = func.params.iter().map(|p| {
            let ident = p.name.to_ident_native(None);

            if p.tref.is_string() {
                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                return quote! { #ident: witx_gen::WasmSlicePtr<u8>, #len_ident: size };
            }

            if let Some(inner) = p.tref.as_array() {
                let inner_type = inner.name.to_ident_native(None);
                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                return quote! { #ident: witx_gen::WasmSlicePtr<#inner_type>, #len_ident: size };
            }

            let tp =  p.tref.to_type();
            quote! { #ident: #tp }
        });

        let param_names = func.params.iter().map(|p| {
            let ident = p.name.to_ident_native(None);

            if p.tref.is_string() {
                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                return quote! { #ident, #len_ident };
            }

            if let Some(_) = p.tref.as_array() {
                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                return quote! { #ident, #len_ident };
            }

            quote! { #ident }
        });

        let results = func.results.iter().map(|p| {
            let typename = p.tref.to_type();

            quote! { #typename }
        });

        quote! {
            fn #ident(&self, ctx: &mut witx_gen::reexports::Ctx, #( #params ),*) -> ( #( #results ),* ) {
                (**self).#ident(ctx, #( #param_names ),*)
            }
        }
    });

    let wasi_trait_impls = wasi_module.funcs().map(|func| {
        let name = func.name.as_str();
        let ident = func.name.to_ident_native(None);

        let param_names = func
            .params
            .iter()
            .map(|p| {
                let ident = p.name.to_ident_native(None);

                if p.tref.is_string() {
                    let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                    return quote! { #ident, #len_ident };
                }

                if let Some(_) = p.tref.as_array() {
                    let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                    return quote! { #ident, #len_ident };
                }

                ident.to_token_stream()
            });

            let params = func.params.iter().map(|p| {
                let ident = p.name.to_ident_native(None);

                if let witx::TypeRef::Value(ref tp) = p.tref {
                    if let witx::Type::Builtin(ref builtin_type) = **tp {
                        if let witx::BuiltinType::String = builtin_type {
                            let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                            return quote! { #ident: witx_gen::WasmSlicePtr<u8>, #len_ident: size };
                        }
                    }
                }

                if let witx::TypeRef::Name(ref named) = p.tref {
                    if let witx::TypeRef::Value(ref tp) = named.tref {
                        if let witx::Type::Array(ref inner) = **tp {
                            if let witx::TypeRef::Name(ref inner_named) = inner {
                                let inner_type = inner_named.name.to_ident_native(None);

                                let len_ident = format!("{}_len", p.name.as_str()).to_ident_native(None);

                                return quote! { #ident: witx_gen::WasmSlicePtr<#inner_type>, #len_ident: size };
                            }
                        }
                    }
                }

                let tp =  p.tref.to_type();
                quote! { #ident: #tp }
            });

        let extra_params = func.results.iter().enumerate().skip(1).map(|(i, p)| {
            let tp = p.tref.to_type();
            let ident = format!("ret_{}", i).to_ident_native(None);

            quote! { #ident: witx_gen::WasmValuePtr<#tp> }
        });

        let extra_results = (1..func.results.len()).map(|i| {
            let ident = format!("ret_{}", i).to_ident_native(None);
            let i = Index::from(i);

            quote! {
                #ident.write(ctx.memory(0), result.#i);
            }
        });

        let result_return = match func.results.len() {
            0 => proc_macro2::TokenStream::new(),
            1 => quote! { result },
            _ => quote! { result.0 },
        };

        quote! {
            #name => witx_gen::reexports::func!({
                let this = self.clone();

                move |ctx: &mut witx_gen::reexports::Ctx, #( #params, )* #( #extra_params, )*| {
                    let result = this.#ident(ctx, #( #param_names ),*);
                    #( #extra_results )*
                    #result_return
                }
            })
        }
    });

    output.extend_native(quote! {
        /// Functions necessary to satisfy the WASI specification.
        pub trait NativeWasiImports: Send + Sync + 'static {
            #( #wasi_trait_functions )*
        }

        /// Extension methods for the [`NativeWasiImports`](trait.NativeWasiImports.html) trait.
        pub trait NativeWasiImportsExt {
            /// Generates the imports for this object.
            fn into_imports(self) -> witx_gen::reexports::ImportObject;
        }

        impl<T: NativeWasiImports> NativeWasiImports for std::sync::Arc<T> {
            #( #wasi_trait_impls_arc )*
        }

        impl<T: NativeWasiImports + Clone> NativeWasiImportsExt for T {
            fn into_imports(self) -> witx_gen::reexports::ImportObject {
                witx_gen::reexports::imports! {
                    #version => {
                        #( #wasi_trait_impls, )*
                    },
                }
            }
        }
    });

    output.into_token_stream().into()

    /*
    let output = output.into_token_stream().into();
    {
        use std::{fs::File, io::Write};

        let mut file = File::create("macro_debug.rs").unwrap();
        write!(file, "{}", output).unwrap();
    }

    output
    */
}
