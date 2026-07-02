use std::ops::Not;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{Error, FnArg, Ident, Token, TraitItemFn, braced};

syn::custom_keyword!(with);

struct AdtType {
    name: Ident,
    type_list: Vec<Ident>,
    trait_def: Option<AdtTraitType>,
}

struct AdtTraitType {
    name: Ident,
    func_list: Vec<TraitItemFn>,
}

impl AdtTraitType {
    fn check_receiver(funcs: &Vec<TraitItemFn>) -> bool {
        funcs.iter().all(|f| {
            f.sig
                .receiver()
                .filter(|&x| x.reference.is_some() && x.mutability.is_none())
                .is_some()
        })
    }
}

impl Parse for AdtTraitType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;

        let body;
        braced!(body in input);

        let mut func_list: Vec<TraitItemFn> = vec![];

        while body.is_empty().not() {
            func_list.push(body.parse::<TraitItemFn>()?);
        }

        if Self::check_receiver(&func_list) {
            Ok(AdtTraitType { name, func_list })
        } else {
            Err(Error::new(
                input.span(),
                "invalid receiver. support only '&self'",
            ))
        }
    }
}

impl Parse for AdtType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;

        let mut type_list: Vec<Ident> = vec![];
        let mut trait_def = None;

        type_list.push(input.parse::<Ident>()?);

        while input.is_empty().not() {
            if input.peek(with) {
                input.parse::<with>()?;

                let att = AdtTraitType::parse(input)?;

                if att.func_list.is_empty().not() {
                    trait_def = Some(att);
                }

                break;
            }

            input.parse::<Token![|]>()?;
            type_list.push(input.parse::<Ident>()?);
        }

        if type_list.len() >= 2 {
            Ok(Self {
                name,
                type_list,
                trait_def,
            })
        } else {
            Err(Error::new(input.span(), "must 2 data types or more"))
        }
    }
}

pub fn adt_proc(input: TokenStream) -> Result<TokenStream> {
    let AdtType {
        name,
        type_list,
        trait_def,
    } = syn::parse2::<AdtType>(input)?;

    let mut elements = TokenStream::new();
    let mut from_impls = TokenStream::new();

    for x in &type_list {
        let enum_type = format_ident!("{}_", x);

        elements = quote! {
            #elements
            #enum_type(#x),
        };

        from_impls = quote! {
            #from_impls

            impl From<#x> for #name {
                fn from(v: #x) -> Self {
                    Self::#enum_type(v)
                }
            }

            impl TryFrom<#name> for #x {
                type Error = ();

                fn try_from(v: #name) -> Result<Self, Self::Error> {
                    if let #name::#enum_type(x) = v {
                        Ok(x)
                    } else {
                        Err(())
                    }
                }
            }
        };
    }

    let mut trait_gen = TokenStream::new();

    if let Some(tr) = trait_def {
        let trait_name = tr.name;

        let mut trait_func = TokenStream::new();
        let mut trait_impl = TokenStream::new();

        for f in tr.func_list {
            trait_func = quote! {
                #trait_func
                #f
            };

            let func_sig = f.sig;
            let func_name = &func_sig.ident;

            let func_args = func_sig.inputs.iter().skip(1).fold(quote! { x }, |acc, x| {
                if let FnArg::Typed(t) = x {
                    let v = &t.pat;

                    quote! {
                        #acc, #v
                    }
                } else {
                    acc
                }
            });

            let func_body = type_list.iter().fold(TokenStream::new(), |acc, x| {
                let enum_type = format_ident!("{}_", x);

                quote! {
                    #acc
                    Self::#enum_type(x) => #trait_name::#func_name(#func_args),
                }
            });

            trait_impl = quote! {
                #trait_impl

                #func_sig {
                    match self {
                        #func_body
                    }
                }
            }
        }

        trait_gen = quote! {
            pub trait #trait_name {
                #trait_func
            }

            impl #trait_name for #name {
                #trait_impl
            }
        };
    }

    Ok(quote! {
        #[derive(Clone, Debug)]
        pub enum #name {
            #elements
        }

        #trait_gen
        #from_impls
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    #[test]
    fn single_type() {
        let input = quote! { Data = Data1 };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn two_types() {
        let input = quote! { Data = Data1 | Data2 };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.type_list.len());

            assert_eq!("Data1", a.type_list.get(0).unwrap().to_string());
            assert_eq!("Data2", a.type_list.get(1).unwrap().to_string());

            assert!(a.trait_def.is_none());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn many_types() {
        let input = quote! { Data = Data1 | Data2 | Data3 | Data4 };

        let r = syn::parse2::<AdtType>(input);

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
    fn with_single_func_no_trait_name() {
        let input = quote! {
            Data = Data1 | Data2 with {
                fn func1(&self, p: isize) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn with_single_func() {
        let input = quote! {
            Data = Data1 | Data2 with DataImpl {
                fn func1(&self, p: isize) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.type_list.len());

            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!("DataImpl", tr.name.to_string());

            assert_eq!(1, tr.func_list.len());

            assert_eq!(
                quote! { fn func1(&self, p: isize) -> String; }.to_string(),
                tr.func_list.get(0).unwrap().to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn with_two_funcs() {
        let input = quote! {
            Data = Data1 | Data2 with DataFunc {
                fn func1(&self, p: isize) -> String;
                fn func2(&self, s: String, b: bool) -> Self;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.type_list.len());

            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!("DataFunc", tr.name.to_string());
            assert_eq!(2, tr.func_list.len());

            assert_eq!(
                quote! { fn func1(&self, p: isize) -> String; }.to_string(),
                tr.func_list.get(0).unwrap().to_token_stream().to_string()
            );
            assert_eq!(
                quote! { fn func2(&self, s: String, b: bool) -> Self; }.to_string(),
                tr.func_list.get(1).unwrap().to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn single_type_with_func() {
        let input = quote! {
            Data = Data1 with DataImpl {
                fn func1(&self) -> Self;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn with_empty_func() {
        let input = quote! {
            Data = Data1 | Data2 with DataImpl {
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.type_list.len());

            assert!(a.trait_def.is_none());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn with_include_noself_func() {
        let input = quote! {
            Data = Data1 | Data2 with DataFunc {
                fn func1(&self, p: isize) -> String;
                fn func2(s: String) -> Self;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn with_include_mut_self_func() {
        let input = quote! {
            Data = Data1 | Data2 with DataImpl {
                fn func1(&self, p: isize) -> String;
                fn func2(&mut self) -> Self;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn with_include_owned_self_func() {
        let input = quote! {
            Data = Data1 | Data2 with DataImpl {
                fn func1(&self, p: isize) -> String;
                fn func2(self, a: String, b: bool) -> Self;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
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

                    impl From<Elem1> for Data {
                        fn from(v: Elem1) -> Self {
                            Self::Elem1_(v)
                        }
                    }

                    impl TryFrom<Data> for Elem1 {
                        type Error = ();

                        fn try_from(v: Data) -> Result<Self, Self::Error> {
                            if let Data::Elem1_(x) = v {
                                Ok(x)
                            } else {
                                Err(())
                            }
                        }
                    }

                    impl From<Elem2> for Data {
                        fn from(v: Elem2) -> Self {
                            Self::Elem2_(v)
                        }
                    }

                    impl TryFrom<Data> for Elem2 {
                        type Error = ();

                        fn try_from(v: Data) -> Result<Self, Self::Error> {
                            if let Data::Elem2_(x) = v {
                                Ok(x)
                            } else {
                                Err(())
                            }
                        }
                    }
                }
                .to_string(),
                t.to_string()
            );
        } else {
            assert!(false, "failed adt_proc")
        }
    }

    #[test]
    fn adt_proc_with_void_func() {
        let input = quote! {
            Data = Elem1 | Elem2 with DataImpl {
                fn func1(&self, a: isize, b: String);
            }
        };

        let r = adt_proc(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
                    #[derive(Clone, Debug)]
                    pub enum Data {
                        Elem1_(Elem1),
                        Elem2_(Elem2),
                    }

                    pub trait DataImpl {
                        fn func1(&self, a: isize, b: String);
                    }

                    impl DataImpl for Data {
                        fn func1(&self, a: isize, b: String) {
                            match self {
                                Self::Elem1_(x) => DataImpl::func1(x, a, b),
                                Self::Elem2_(x) => DataImpl::func1(x, a, b),
                            }
                        }
                    }

                    impl From<Elem1> for Data {
                        fn from(v: Elem1) -> Self {
                            Self::Elem1_(v)
                        }
                    }

                    impl TryFrom<Data> for Elem1 {
                        type Error = ();

                        fn try_from(v: Data) -> Result<Self, Self::Error> {
                            if let Data::Elem1_(x) = v {
                                Ok(x)
                            } else {
                                Err(())
                            }
                        }
                    }

                    impl From<Elem2> for Data {
                        fn from(v: Elem2) -> Self {
                            Self::Elem2_(v)
                        }
                    }

                    impl TryFrom<Data> for Elem2 {
                        type Error = ();

                        fn try_from(v: Data) -> Result<Self, Self::Error> {
                            if let Data::Elem2_(x) = v {
                                Ok(x)
                            } else {
                                Err(())
                            }
                        }
                    }
                }
                .to_string(),
                t.to_string()
            );
        } else {
            assert!(false)
        }
    }

    #[test]
    fn adt_proc_with_multi_funcs() {
        let input = quote! {
            Data = Elem1 | Elem2 with DataFunc {
                fn func1(&self);
                fn func2(&self, a: isize) -> bool;
            }
        };

        let r = adt_proc(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
                    #[derive(Clone, Debug)]
                    pub enum Data {
                        Elem1_(Elem1),
                        Elem2_(Elem2),
                    }

                    pub trait DataFunc {
                        fn func1(&self);
                        fn func2(&self, a: isize) -> bool;
                    }

                    impl DataFunc for Data {
                        fn func1(&self) {
                            match self {
                                Self::Elem1_(x) => DataFunc::func1(x),
                                Self::Elem2_(x) => DataFunc::func1(x),
                            }
                        }

                        fn func2(&self, a: isize) -> bool {
                            match self {
                                Self::Elem1_(x) => DataFunc::func2(x, a),
                                Self::Elem2_(x) => DataFunc::func2(x, a),
                            }
                        }
                    }

                    impl From<Elem1> for Data {
                        fn from(v: Elem1) -> Self {
                            Self::Elem1_(v)
                        }
                    }

                    impl TryFrom<Data> for Elem1 {
                        type Error = ();

                        fn try_from(v: Data) -> Result<Self, Self::Error> {
                            if let Data::Elem1_(x) = v {
                                Ok(x)
                            } else {
                                Err(())
                            }
                        }
                    }

                    impl From<Elem2> for Data {
                        fn from(v: Elem2) -> Self {
                            Self::Elem2_(v)
                        }
                    }

                    impl TryFrom<Data> for Elem2 {
                        type Error = ();

                        fn try_from(v: Data) -> Result<Self, Self::Error> {
                            if let Data::Elem2_(x) = v {
                                Ok(x)
                            } else {
                                Err(())
                            }
                        }
                    }
                }
                .to_string(),
                t.to_string()
            );
        } else {
            assert!(false)
        }
    }
}
