use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Error, Expr, Ident, Token,
    parse::{Parse, ParseStream, Result},
};

use std::ops::Not;

pub fn mdo_generate(input: TokenStream) -> Result<TokenStream> {
    let MdoBlock { stmts } = syn::parse2::<MdoBlock>(input)?;

    let mut body = TokenStream::new();

    for s in stmts.into_iter().rev() {
        body = match s {
            MdoStmt::Bind(MdoBind { var, expr }) => {
                let v = var.map(|x| x.to_token_stream()).unwrap_or(quote! { _ });

                quote! { (#expr).bind(|#v| #body) }
            }
            MdoStmt::Yield(MdoYield { expr }) => quote! { MonadLike::unit(#expr) #body },
        };
    }

    Ok(body)
}

struct MdoBlock {
    stmts: Vec<MdoStmt>,
}

enum MdoStmt {
    Bind(MdoBind),
    Yield(MdoYield),
}

struct MdoBind {
    var: Option<Ident>,
    expr: Expr,
}

struct MdoYield {
    expr: Expr,
}

impl Parse for MdoBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stmts = Vec::new();

        while input.is_empty().not() {
            stmts.push(input.parse::<MdoStmt>()?);
        }

        if stmts.is_empty() {
            return Err(Error::new(input.span(), "empty block"));
        } else if let Some(x) = stmts.last()
            && x.is_yield().not()
        {
            return Err(Error::new(input.span(), "must end with a yield"));
        }

        Ok(Self { stmts })
    }
}

impl MdoStmt {
    fn is_yield(&self) -> bool {
        match self {
            Self::Yield(_) => true,
            _ => false,
        }
    }
}

impl Parse for MdoStmt {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![yield]) {
            input.parse::<MdoYield>().map(Self::Yield)
        } else {
            input.parse::<MdoBind>().map(Self::Bind)
        }
    }
}

impl Parse for MdoBind {
    fn parse(input: ParseStream) -> Result<Self> {
        let var = if input.peek(Token![_]) {
            input.parse::<Token![_]>()?;
            None
        } else {
            Some(input.parse::<Ident>()?)
        };

        input.parse::<Token![<]>()?;
        input.parse::<Token![-]>()?;

        let expr = input.parse::<Expr>()?;

        Ok(Self { var, expr })
    }
}

impl Parse for MdoYield {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![yield]>()?;

        let expr = input.parse::<Expr>()?;

        Ok(Self { expr })
    }
}

#[cfg(test)]
mod tests {
    use quote::ToTokens;

    use super::*;

    #[test]
    fn parse_bind_vec() {
        let input = quote! { x <- vec![1, 2] };

        if let Ok(x) = syn::parse2::<MdoBind>(input) {
            assert!(x.var.is_some());
            assert_eq!("x", x.var.unwrap().to_string());
            assert_eq!(
                quote! { vec![1, 2] }.to_string(),
                x.expr.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_bind_variable() {
        let input = quote! { x <- a };

        if let Ok(x) = syn::parse2::<MdoBind>(input) {
            assert_eq!("x", x.var.unwrap().to_string());
            assert_eq!("a", x.expr.to_token_stream().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_bind_variable_underscore() {
        let input = quote! { _ <- a };

        if let Ok(x) = syn::parse2::<MdoBind>(input) {
            assert!(x.var.is_none());
            assert_eq!("a", x.expr.to_token_stream().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_yield_single() {
        let input = quote! { yield x };

        if let Ok(x) = syn::parse2::<MdoYield>(input) {
            assert_eq!("x", x.expr.to_token_stream().to_string());
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_yield_tuple() {
        let input = quote! { yield (a, b, true) };

        if let Ok(x) = syn::parse2::<MdoYield>(input) {
            assert_eq!(
                quote! { (a, b, true) }.to_string(),
                x.expr.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_stmt_bind() {
        let input = quote! { x <- vec![1, 2] };

        if let Ok(MdoStmt::Bind(x)) = syn::parse2::<MdoStmt>(input) {
            assert_eq!("x", x.var.unwrap().to_string());
            assert_eq!(
                quote! { vec![1, 2] }.to_string(),
                x.expr.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_stmt_yield() {
        let input = quote! { yield (a, b, true) };

        if let Ok(MdoStmt::Yield(x)) = syn::parse2::<MdoStmt>(input) {
            assert_eq!(
                quote! { (a, b, true) }.to_string(),
                x.expr.to_token_stream().to_string()
            );
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn parse_block() {
        let input = quote! {
            x <- a
            y <- vec![1, 2, 3]
            yield (x, y, true)
        };

        if let Ok(x) = syn::parse2::<MdoBlock>(input) {
            assert_eq!(3, x.stmts.len());

            if let Some(MdoStmt::Bind(b)) = x.stmts.first() {
                assert_eq!("x", b.var.clone().unwrap().to_string());
            } else {
                assert!(false, "ummatch bind")
            }

            if let Some(MdoStmt::Yield(y)) = x.stmts.last() {
                assert_eq!(
                    quote! { (x, y, true) }.to_string(),
                    y.expr.to_token_stream().to_string()
                );
            } else {
                assert!(false, "ummatch yield")
            }
        } else {
            assert!(false, "parse error")
        }
    }

    #[test]
    fn parse_block_empty() {
        let input = quote! {};

        let r = syn::parse2::<MdoBlock>(input);

        assert!(r.is_err());
    }

    #[test]
    fn parse_block_with_bind_end() {
        let input = quote! {
            x <- a
            yield (x, 1)
            y <- b
        };

        let r = syn::parse2::<MdoBlock>(input);

        assert!(r.is_err());
    }

    #[test]
    fn generate_single() {
        let input = quote! {
            x <- vec![1, 2]
            yield x + 1
        };

        let r = mdo_generate(input);

        if let Ok(x) = r {
            assert_eq!(
                quote! {
                    (vec![1, 2]).bind(|x| MonadLike::unit(x + 1))
                }
                .to_string(),
                x.to_string()
            );
        } else {
            assert!(false, "generate error")
        }
    }

    #[test]
    fn generate_nest() {
        let input = quote! {
            x <- vec![1, 2]
            y <- vec!["a", "b", "c"]
            z <- vec![true, false]
            yield (x * 2, y, z)
        };

        let r = mdo_generate(input);

        if let Ok(x) = r {
            assert_eq!(
                quote! {
                    (vec![1, 2]).bind(|x|
                        (vec!["a", "b", "c"]).bind(|y|
                            (vec![true, false]).bind(|z| MonadLike::unit((x * 2, y, z)))
                        )
                    )
                }
                .to_string(),
                x.to_string()
            );
        } else {
            assert!(false, "generate error")
        }
    }

    #[test]
    fn generate_nest_underscore() {
        let input = quote! {
            x <- vec![1, 2]
            _ <- vec!["a", "b", "c"]
            z <- vec![true, false]
            yield (x * 2, z)
        };

        let r = mdo_generate(input);

        if let Ok(x) = r {
            assert_eq!(
                quote! {
                    (vec![1, 2]).bind(|x|
                        (vec!["a", "b", "c"]).bind(|_|
                            (vec![true, false]).bind(|z| MonadLike::unit((x * 2, z)))
                        )
                    )
                }
                .to_string(),
                x.to_string()
            );
        } else {
            assert!(false, "generate error")
        }
    }
}
