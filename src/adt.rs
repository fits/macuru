use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Error, Ident, Token};

#[derive(Debug)]
struct AdtType {
    name: Ident,
    type_list: Vec<Ident>,
}

impl Parse for AdtType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;

        let mut type_list: Vec<Ident> = vec![];

        type_list.push(input.parse::<Ident>()?);

        while !input.is_empty() {
            input.parse::<Token![|]>()?;
            type_list.push(input.parse::<Ident>()?);
        }

        if type_list.len() >= 2 {
            Ok(Self { name, type_list })
        } else {
            Err(Error::new(input.span(), "must 2 data types or more"))
        }
    }
}

pub fn adt_proc(input: TokenStream) -> Result<TokenStream> {
    let AdtType { name, type_list } = syn::parse2::<AdtType>(input)?;

    let elements = type_list.iter().fold(TokenStream::new(), |acc, x| {
        let enum_type = format_ident!("{}_", x);

        quote! {
            #acc
            #enum_type(#x),
        }
    });

    Ok(quote! {
        #[derive(Clone, Debug)]
        pub enum #name {
            #elements
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_type() {
        let input = quote! { Data = Data1 };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn two_type() {
        let input = quote! { Data = Data1 | Data2 };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.type_list.len());

            assert_eq!("Data1", a.type_list.get(0).unwrap().to_string());
            assert_eq!("Data2", a.type_list.get(1).unwrap().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn many_type() {
        let input = quote! { Data = Data1 | Data2 | Data3 | Data4 };

        let r = syn::parse2::<AdtType>(input);

        println!("{:?}", r);

        assert!(r.is_ok());
    }

    #[test]
    fn lacked_type() {
        let r1 = syn::parse2::<AdtType>(quote! { Data = Data1 | });
        assert!(r1.is_err());

        let r2 = syn::parse2::<AdtType>(quote! { Data = Data1 | Data2 | });
        assert!(r2.is_err());
    }

    #[test]
    fn adt_proc_with_two_types() {
        let input = quote! { Data = Elem1 | Elem2 };

        let r = adt_proc(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
                    #[derive(Clone, Debug)]
                    pub enum Data {
                        Elem1_(Elem1),
                        Elem2_(Elem2),
                    }
                }
                .to_string(),
                t.to_string()
            );
        } else {
            assert!(false, "failed adt_proc")
        }
    }
}
