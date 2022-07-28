#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Either<L, R> {
    /// A value of type `L`.
    Left(L),
    /// A value of type `R`.
    Right(R),
}

use self::Either::*;

impl<L: Clone, R: Clone> Clone for Either<L, R> {
    fn clone(&self) -> Self {
        match self {
            Left(inner) => Left(inner.clone()),
            Right(inner) => Right(inner.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Left(dest), Left(source)) => dest.clone_from(source),
            (Right(dest), Right(source)) => dest.clone_from(source),
            (dest, source) => *dest = source.clone(),
        }
    }
}
