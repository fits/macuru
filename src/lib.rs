use proc_macro::TokenStream;

mod adt;

use adt::adt_proc;

#[proc_macro]
pub fn adt(input: TokenStream) -> TokenStream {
    adt_proc(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
