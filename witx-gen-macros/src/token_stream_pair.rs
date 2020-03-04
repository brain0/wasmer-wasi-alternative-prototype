use proc_macro2::TokenStream;
use quote::quote;

pub(crate) struct TokenStreamPair {
    native: TokenStream,
    mapped: TokenStream,
}

impl TokenStreamPair {
    pub(crate) fn new() -> TokenStreamPair {
        Self::from_streams(TokenStream::new(), TokenStream::new())
    }

    pub(crate) fn from_streams(native: TokenStream, mapped: TokenStream) -> TokenStreamPair {
        TokenStreamPair { native, mapped }
    }

    pub(crate) fn extend(mut self, other: TokenStreamPair) -> Self {
        self.native.extend(other.native);
        self.mapped.extend(other.mapped);
        self
    }

    pub(crate) fn extend_native(&mut self, other: TokenStream) {
        self.native.extend(other);
    }

    pub(crate) fn into_token_stream(self) -> TokenStream {
        let Self { native, mapped } = self;

        quote! {
            #[allow(nonstandard_style)]
            pub mod native {
                #native

                #[doc(hidden)]
                #[derive(Copy, Clone, Debug)]
                pub struct Private(());
            }
            #mapped
        }
    }
}
