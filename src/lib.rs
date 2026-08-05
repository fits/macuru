pub use macuru_macros::*;

pub trait MonadLike<A> {
    type Target<B>: MonadLike<B>;

    fn unit(value: A) -> Self;

    fn bind<F, B>(self, f: F) -> Self::Target<B>
    where
        F: Fn(A) -> Self::Target<B>;
}

impl<A> MonadLike<A> for Option<A> {
    type Target<B> = Option<B>;

    fn unit(value: A) -> Self {
        Some(value)
    }

    fn bind<F, B>(self, f: F) -> Self::Target<B>
    where
        F: Fn(A) -> Self::Target<B>,
    {
        self.and_then(f)
    }
}

impl<A, E> MonadLike<A> for std::result::Result<A, E> {
    type Target<B> = std::result::Result<B, E>;

    fn unit(value: A) -> Self {
        Ok(value)
    }

    fn bind<F, B>(self, f: F) -> Self::Target<B>
    where
        F: Fn(A) -> Self::Target<B>,
    {
        self.and_then(f)
    }
}

impl<A> MonadLike<A> for Vec<A> {
    type Target<B> = Vec<B>;

    fn unit(value: A) -> Self {
        vec![value]
    }

    fn bind<F, B>(self, f: F) -> Self::Target<B>
    where
        F: Fn(A) -> Self::Target<B>,
    {
        self.into_iter().flat_map(f).collect()
    }
}
