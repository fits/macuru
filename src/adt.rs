use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::fold::Fold;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Error, FnArg, Generics, Ident, Signature, Token, TraitItemFn, WhereClause, braced};

use std::ops::Not;

syn::custom_keyword!(derive);
syn::custom_keyword!(with);

struct AdtType {
    name: Ident,
    generics: Option<Generics>,
    elements: Vec<ElementType>,
    derive_def: Option<AdtDeriveType>,
    trait_def: Option<AdtTraitType>,
}

struct AdtDeriveType {
    derives: Punctuated<Ident, Token![,]>,
}

struct AdtTraitType {
    name: Ident,
    generics: Option<Generics>,
    where_clause: Option<WhereClause>,
    functions: Vec<TraitItemFn>,
}

struct ElementType {
    ident: Ident,
    type_param: Option<ElementTypeParam>,
}

struct ElementTypeParam {
    lt_token: Token![<],
    params: Punctuated<Ident, Token![,]>,
    gt_token: Token![>],
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

impl Parse for AdtType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;

        let generics = if input.peek(Token![<]) {
            Some(input.parse::<Generics>()?)
        } else {
            None
        };

        input.parse::<Token![=]>()?;

        let mut elements = vec![];

        let mut derive_def = None;
        let mut trait_def = None;

        elements.push(input.parse::<ElementType>()?);

        while input.is_empty().not() {
            if input.peek(derive) {
                input.parse::<derive>()?;

                derive_def = Some(AdtDeriveType::parse(input)?);

                if input.is_empty() {
                    break;
                }
            }

            if input.peek(with) {
                input.parse::<with>()?;

                let att = AdtTraitType::parse(input)?;

                if att.functions.is_empty().not() {
                    trait_def = Some(att);
                }

                break;
            }

            input.parse::<Token![|]>()?;
            elements.push(input.parse::<ElementType>()?);
        }

        if elements.len() >= 2 {
            Ok(Self {
                name,
                generics,
                elements,
                derive_def,
                trait_def,
            })
        } else {
            Err(Error::new(input.span(), "must 2 elements or more"))
        }
    }
}

impl Parse for AdtDeriveType {
    fn parse(input: ParseStream) -> Result<Self> {
        let derives = parse_punct_idents(&input)?;

        Ok(Self { derives })
    }
}

impl Parse for AdtTraitType {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;

        let generics = if input.peek(Token![<]) {
            Some(input.parse::<Generics>()?)
        } else {
            None
        };

        let where_clause = if input.peek(Token![where]) {
            Some(input.parse::<WhereClause>()?)
        } else {
            None
        };

        let body;
        braced!(body in input);

        let mut functions: Vec<TraitItemFn> = vec![];

        while body.is_empty().not() {
            functions.push(body.parse::<TraitItemFn>()?);
        }

        if Self::check_receiver(&functions) {
            Ok(AdtTraitType {
                name,
                generics,
                where_clause,
                functions,
            })
        } else {
            Err(Error::new(
                input.span(),
                "invalid receiver. support only '&self'",
            ))
        }
    }
}

impl Parse for ElementType {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;

        let type_param = if input.peek(Token![<]) {
            Some(input.parse::<ElementTypeParam>()?)
        } else {
            None
        };

        Ok(Self { ident, type_param })
    }
}

impl Parse for ElementTypeParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let lt_token = input.parse::<Token![<]>()?;
        let params = parse_punct_idents(&input)?;
        let gt_token = input.parse::<Token![>]>()?;

        Ok(Self {
            lt_token,
            params,
            gt_token,
        })
    }
}

impl ToTokens for ElementType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);

        if let Some(x) = &self.type_param {
            x.to_tokens(tokens);
        }
    }
}

impl ToTokens for ElementTypeParam {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.lt_token.to_tokens(tokens);
        self.params.to_tokens(tokens);
        self.gt_token.to_tokens(tokens);
    }
}

struct SelfTypeEditor(Ident);

impl Fold for SelfTypeEditor {
    fn fold_path_segment(&mut self, i: syn::PathSegment) -> syn::PathSegment {
        if i.ident.to_string() == "Self" {
            let mut res = i.clone();
            res.ident = self.0.clone();
            res
        } else {
            syn::fold::fold_path_segment(self, i)
        }
    }
}

pub fn adt_generate(input: TokenStream) -> Result<TokenStream> {
    let AdtType {
        name,
        generics,
        elements,
        derive_def,
        trait_def,
    } = syn::parse2::<AdtType>(input)?;

    let mut elements_def = TokenStream::new();
    let mut from_impls = TokenStream::new();

    for x in &elements {
        let enum_element = to_element_name(&x.ident);

        elements_def = quote! {
            #elements_def
            #enum_element(#x),
        };

        from_impls = quote! {
            #from_impls

            impl From<#x> for #name {
                fn from(v: #x) -> Self {
                    Self::#enum_element(v)
                }
            }

            impl TryFrom<#name> for #x {
                type Error = ();

                fn try_from(v: #name) -> Result<Self, Self::Error> {
                    if let #name::#enum_element(x) = v {
                        Ok(x)
                    } else {
                        Err(())
                    }
                }
            }
        };
    }

    let derive_gen = derive_def.map(|x| derive_generate(x)).unwrap_or_default();

    let trait_gen = trait_def
        .map(|x| trait_generate(&name, &elements, x))
        .unwrap_or_default();

    Ok(quote! {
        #derive_gen
        pub enum #name {
            #elements_def
        }

        #trait_gen
        #from_impls
    })
}

fn parse_punct_idents(input: &ParseStream) -> Result<Punctuated<Ident, Token![,]>> {
    let mut res = Punctuated::new();

    res.push_value(input.parse::<Ident>()?);

    while input.is_empty().not() && input.peek(Token![,]) {
        res.push_punct(input.parse::<Token![,]>()?);
        res.push_value(input.parse::<Ident>()?);
    }

    Ok(res)
}

fn to_element_name(inner_type: &Ident) -> Ident {
    format_ident!("{}_", inner_type)
}

fn edit_self_return_type(sig: &Signature, replace_name: &Ident) -> Signature {
    let mut res = sig.clone();

    let mut editor = SelfTypeEditor(replace_name.clone());

    res.output = editor.fold_return_type(res.output);

    res
}

fn derive_generate(dt: AdtDeriveType) -> TokenStream {
    let derive_args = dt.derives;
    quote! { #[derive(#derive_args)] }
}

fn trait_generate(name: &Ident, elements: &Vec<ElementType>, tt: AdtTraitType) -> TokenStream {
    let trait_name = tt.name;

    let mut trait_func = TokenStream::new();
    let mut trait_impl = TokenStream::new();

    for f in tt.functions {
        let mut f = f.clone();
        f.sig = edit_self_return_type(&f.sig, name);

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

        let func_body = elements.iter().fold(TokenStream::new(), |acc, x| {
            let enum_element = to_element_name(&x.ident);

            quote! {
                #acc
                Self::#enum_element(x) => #trait_name::#func_name(#func_args),
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

    quote! {
        pub trait #trait_name {
            #trait_func
        }

        impl #trait_name for #name {
            #trait_impl
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    fn parse_func(input: TokenStream) -> Signature {
        syn::parse2(input).unwrap()
    }

    #[test]
    fn element_typeparam() {
        let input = quote! { <A, B, C> };

        if let Ok(x) = syn::parse2::<ElementTypeParam>(input) {
            assert_eq!(3, x.params.len());
            assert_eq!(
                quote! {<A, B, C>}.to_string(),
                x.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn element_typeparam_last_comma() {
        let input = quote! { <A, B,> };

        let r = syn::parse2::<ElementTypeParam>(input);

        assert!(r.is_err());
    }

    #[test]
    fn element_typeparam_restrict() {
        let input = quote! { <A, B, C: Clone> };

        let r = syn::parse2::<ElementTypeParam>(input);

        assert!(r.is_err());
    }

    #[test]
    fn element_basic() {
        let input = quote! { Elem1 };

        if let Ok(x) = syn::parse2::<ElementType>(input) {
            assert_eq!("Elem1", x.ident.to_string());
            assert!(x.type_param.is_none());

            assert_eq!(quote! {Elem1}.to_string(), x.to_token_stream().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn element_generics() {
        let input = quote! { Elem1<A, B> };

        if let Ok(x) = syn::parse2::<ElementType>(input) {
            assert_eq!("Elem1", x.ident.to_string());
            assert!(x.type_param.is_some());

            assert_eq!(
                quote! {Elem1<A, B>}.to_string(),
                x.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

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
            assert_eq!(2, a.elements.len());

            assert_eq!(
                "Data1",
                a.elements.get(0).unwrap().to_token_stream().to_string()
            );
            assert_eq!(
                "Data2",
                a.elements.get(1).unwrap().to_token_stream().to_string()
            );

            assert!(a.generics.is_none());
            assert!(a.derive_def.is_none());
            assert!(a.trait_def.is_none());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn generics_enum() {
        let input = quote! { Data<T> = Elem1 | Elem2 };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_ok());

        let t = r.unwrap();

        if let Some(g) = t.generics {
            assert_eq!(1, g.params.len());
            assert_eq!(quote! { <T> }.to_string(), g.to_token_stream().to_string());
        } else {
            assert!(false, "none generics")
        }
    }

    #[test]
    fn generics_enum_type_params() {
        let input = quote! { Data<A, B, C: Clone> = Elem1 | Elem2 };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_ok());

        let t = r.unwrap();

        if let Some(g) = t.generics {
            assert_eq!(3, g.params.len());
            assert_eq!(
                quote! { <A, B, C: Clone> }.to_string(),
                g.to_token_stream().to_string()
            );
        } else {
            assert!(false, "none generics")
        }
    }

    #[test]
    fn generics_enum_with_where() {
        let input = quote! {
            Data<A, B> where = Elem1 | Elem2
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn generics_enum_with_where_single() {
        let input = quote! {
            Data<A> where A: Debug = Elem1 | Elem2
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn generics_element() {
        let input = quote! { Data = Elem1<i32> | Elem2 };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!(2, a.elements.len());

            let el1 = a.elements.get(0).unwrap();

            assert_eq!("Elem1", el1.ident.to_string());
            assert_eq!(
                quote! {Elem1<i32>}.to_string(),
                el1.to_token_stream().to_string()
            );

            let el2 = a.elements.get(1).unwrap();
            assert_eq!(
                quote! {Elem2}.to_string(),
                el2.to_token_stream().to_string()
            );
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
    fn single_derive() {
        let input = quote! { Data = Data1 | Data2 derive Clone };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert!(a.derive_def.is_some());

            let d = a.derive_def.unwrap();
            assert_eq!("Clone", d.derives.get(0).unwrap().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn empty_derive() {
        let input = quote! { Data = Data1 | Data2 derive };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn empty_derive_and_with() {
        let input = quote! { Data = Data1 | Data2 derive with DataImpl {}};

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn single_type_derive() {
        let input = quote! {
            Data = Data1 derive Debug
        };

        let r = syn::parse2::<AdtType>(input);

        assert!(r.is_err());
    }

    #[test]
    fn many_derives() {
        let input = quote! { Data = Data1 | Data2 derive Clone, Debug, PartialEq };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert!(a.derive_def.is_some());

            let d = a.derive_def.unwrap();

            assert_eq!(3, d.derives.len());
            assert_eq!("Clone", d.derives.get(0).unwrap().to_string());
            assert_eq!("Debug", d.derives.get(1).unwrap().to_string());
            assert_eq!("PartialEq", d.derives.get(2).unwrap().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn derive_and_with() {
        let input = quote! {
            Data = Data1 | Data2 derive Clone with DataImpl {
                fn func1(&self, p: isize) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.elements.len());
            assert!(a.derive_def.is_some());
            assert_eq!(1, a.derive_def.unwrap().derives.len());
            assert!(a.trait_def.is_some());
        } else {
            assert!(false);
        }
    }

    #[test]
    fn many_derives_and_with() {
        let input = quote! {
            Data = Data1 | Data2 derive Clone, Debug, PartialEq with DataImpl {
                fn func1(&self, p: isize) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.elements.len());
            assert!(a.derive_def.is_some());
            assert_eq!(3, a.derive_def.unwrap().derives.len());
            assert!(a.trait_def.is_some());
        } else {
            assert!(false);
        }
    }

    #[test]
    fn with_generics_trait() {
        let input = quote! {
            Data = Data1 | Data2 with DataFunc<A, B> {
                fn func1(&self, p: T) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!(
                quote! {<A, B>}.to_string(),
                tr.generics.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn with_generics_trait_where() {
        let input = quote! {
            Data = Data1 | Data2 with DataFunc<A, B>
            where
                A: Clone,
                B: Clone + Copy + PartialEq,
            {
                fn func1(&self, p: T) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!(
                quote! {
                    where
                        A: Clone,
                        B: Clone + Copy + PartialEq,
                }
                .to_string(),
                tr.where_clause.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
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
            assert_eq!(2, a.elements.len());

            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!("DataImpl", tr.name.to_string());

            assert_eq!(1, tr.functions.len());

            assert_eq!(
                quote! { fn func1(&self, p: isize) -> String; }.to_string(),
                tr.functions.get(0).unwrap().to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn with_generics_func() {
        let input = quote! {
            Data = Data1 | Data2 with DataImpl {
                fn func1<T>(&self, p: T) -> String;
            }
        };

        let r = syn::parse2::<AdtType>(input);

        if let Ok(a) = r {
            assert_eq!("Data", a.name.to_string());
            assert_eq!(2, a.elements.len());

            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!("DataImpl", tr.name.to_string());

            assert_eq!(1, tr.functions.len());

            assert_eq!(
                quote! { fn func1<T>(&self, p: T) -> String; }.to_string(),
                tr.functions.get(0).unwrap().to_token_stream().to_string()
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
            assert_eq!(2, a.elements.len());

            assert!(a.trait_def.is_some());

            let tr = a.trait_def.unwrap();

            assert_eq!("DataFunc", tr.name.to_string());
            assert_eq!(2, tr.functions.len());

            assert_eq!(
                quote! { fn func1(&self, p: isize) -> String; }.to_string(),
                tr.functions.get(0).unwrap().to_token_stream().to_string()
            );
            assert_eq!(
                quote! { fn func2(&self, s: String, b: bool) -> Self; }.to_string(),
                tr.functions.get(1).unwrap().to_token_stream().to_string()
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
    fn single_type_with_derive_and_func() {
        let input = quote! {
            Data = Data1 derive Debug with DataImpl {
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
            assert_eq!(2, a.elements.len());

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
    fn return_type_with_no_self() {
        let name = format_ident!("TEST");

        let f1 = parse_func(quote! { fn func1() -> bool });

        assert_eq!(
            quote! { fn func1() -> bool }.to_string(),
            edit_self_return_type(&f1, &name)
                .to_token_stream()
                .to_string()
        );

        let f2 = parse_func(quote! { fn func1() -> Option<(bool, String, i32)> });

        assert_eq!(
            quote! { fn func1() -> Option<(bool, String, i32)> }.to_string(),
            edit_self_return_type(&f2, &name)
                .to_token_stream()
                .to_string()
        );
    }

    #[test]
    fn return_type_with_only_self() {
        let name = format_ident!("TEST");

        let f1 = parse_func(quote! { fn func1() -> Self });

        assert_eq!(
            quote! { fn func1() -> TEST }.to_string(),
            edit_self_return_type(&f1, &name)
                .to_token_stream()
                .to_string()
        );
    }

    #[test]
    fn return_type_with_tuple_in_self() {
        let name = format_ident!("TEST");

        let f1 = parse_func(quote! { fn func1() -> (Self,) });

        assert_eq!(
            quote! { fn func1() -> (TEST,) }.to_string(),
            edit_self_return_type(&f1, &name)
                .to_token_stream()
                .to_string()
        );

        let f2 = parse_func(quote! { fn func2() -> (bool, Self, i32) });

        assert_eq!(
            quote! { fn func2() -> (bool, TEST, i32) }.to_string(),
            edit_self_return_type(&f2, &name)
                .to_token_stream()
                .to_string()
        );

        let f3 = parse_func(quote! { fn func3() -> (bool, Self, (i32, String, Self)) });

        assert_eq!(
            quote! { fn func3() -> (bool, TEST, (i32, String, TEST)) }.to_string(),
            edit_self_return_type(&f3, &name)
                .to_token_stream()
                .to_string()
        );
    }

    #[test]
    fn return_type_with_self_in_type() {
        let name = format_ident!("TEST");

        let f1 = parse_func(quote! { fn func1() -> Result<Option<(bool, Self)>, ()> });

        assert_eq!(
            quote! { fn func1() -> Result<Option<(bool, TEST)>, ()> }.to_string(),
            edit_self_return_type(&f1, &name)
                .to_token_stream()
                .to_string()
        );
    }

    #[test]
    fn return_type_with_self_in_fn() {
        let name = format_ident!("TEST");

        let f1 = parse_func(quote! { fn func1() -> impl Fn(i32) -> Option<(bool, Self)> });

        assert_eq!(
            quote! { fn func1() -> impl Fn(i32) -> Option<(bool, TEST)> }.to_string(),
            edit_self_return_type(&f1, &name)
                .to_token_stream()
                .to_string()
        );
    }

    #[test]
    fn adt_generate_with_two_types() {
        let input = quote! { Data = Elem1 | Elem2 };

        let r = adt_generate(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
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
    fn adt_generate_with_two_types_and_derive() {
        let input = quote! { Data = Elem1 | Elem2 derive Clone, Debug };

        let r = adt_generate(input);

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
    fn adt_generate_with_void_func() {
        let input = quote! {
            Data = Elem1 | Elem2 with DataImpl {
                fn func1(&self, a: isize, b: String);
            }
        };

        let r = adt_generate(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
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
    fn adt_generate_with_void_func_and_derive() {
        let input = quote! {
            Data = Elem1 | Elem2 derive Debug, Clone with DataImpl {
                fn func1(&self, a: isize, b: String);
            }
        };

        let r = adt_generate(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
                    #[derive(Debug, Clone)]
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
    fn adt_generate_with_multi_funcs() {
        let input = quote! {
            Data = Elem1 | Elem2 with DataFunc {
                fn func1(&self);
                fn func2(&self, a: isize) -> bool;
            }
        };

        let r = adt_generate(input);

        if let Ok(t) = r {
            assert_eq!(
                quote! {
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

    #[test]
    fn adt_generate_with_self_return_func_and_derive() {
        let input = quote! {
            Data = Elem1 | Elem2 derive Clone, Debug with DataFunc {
                fn func1(&self);
                fn func2(&self, a: isize) -> Self;
                fn func3(&self, a: String, b: bool) -> (Self, isize);
                fn func4(&self, a: f32) -> Result<(Self, String, isize), ()>;
            }
        };

        let r = adt_generate(input);

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
                        fn func2(&self, a: isize) -> Data;
                        fn func3(&self, a: String, b: bool) -> (Data, isize);
                        fn func4(&self, a: f32) -> Result<(Data, String, isize), ()>;
                    }

                    impl DataFunc for Data {
                        fn func1(&self) {
                            match self {
                                Self::Elem1_(x) => DataFunc::func1(x),
                                Self::Elem2_(x) => DataFunc::func1(x),
                            }
                        }

                        fn func2(&self, a: isize) -> Data {
                            match self {
                                Self::Elem1_(x) => DataFunc::func2(x, a),
                                Self::Elem2_(x) => DataFunc::func2(x, a),
                            }
                        }

                        fn func3(&self, a: String, b: bool) -> (Data, isize) {
                            match self {
                                Self::Elem1_(x) => DataFunc::func3(x, a, b),
                                Self::Elem2_(x) => DataFunc::func3(x, a, b),
                            }
                        }

                        fn func4(&self, a: f32) -> Result<(Data, String, isize), ()> {
                            match self {
                                Self::Elem1_(x) => DataFunc::func4(x, a),
                                Self::Elem2_(x) => DataFunc::func4(x, a),
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
