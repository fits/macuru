pub use macuru_macros::*;

pub trait MonadLike<A> {
    type Target<B>: MonadLike<B>;

    fn unit(value: A) -> Self;

    fn bind<F, B>(self, f: F) -> Self::Target<B>
    where
        F: Fn(A) -> Self::Target<B>;

    fn filter<P>(self, _pred: P) -> Self
    where
        Self: Sized,
        P: Fn(&A) -> bool,
    {
        self
    }
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

    fn filter<P>(self, pred: P) -> Self
    where
        P: Fn(&A) -> bool,
    {
        self.filter(pred)
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

    fn filter<P>(self, pred: P) -> Self
    where
        P: Fn(&A) -> bool,
    {
        self.into_iter().filter(pred).collect()
    }
}
