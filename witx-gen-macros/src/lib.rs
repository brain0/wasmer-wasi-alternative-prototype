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
mod interfaces;
mod pointer;
mod string_ext;
mod struct_type;
mod to_ident;
mod token_stream_pair;
mod type_definition_extensions;
mod type_ref_ext;
mod union;

use self::{
    interfaces::generate_interfaces, string_ext::StringExt, token_stream_pair::TokenStreamPair,
    type_definition_extensions::TypeDefinitionExtensions,
};
use proc_macro::TokenStream;
use std::{env, path::Path};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::FatArrow,
    LitStr,
};

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

    output.extend_native(generate_interfaces(&document, &version));

    output.into_token_stream().into()
}
