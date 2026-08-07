use proc_macro::TokenStream;

mod adt;
mod mdo;

use adt::adt_generate;
use mdo::mdo_generate;

#[proc_macro]
pub fn adt(input: TokenStream) -> TokenStream {
    adt_generate(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro]
pub fn mdo(input: TokenStream) -> TokenStream {
    mdo_generate(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
