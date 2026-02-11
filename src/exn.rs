//! [exn](https://crates.io/crates/exn) inspired error handling extensions

use std::{convert::Infallible, error::Error};

pub(crate) trait Exn: Error {
    fn with_source(self, err: impl Error + Send + Sync + 'static) -> Self;
}

pub(crate) trait ResultExt {
    type Success;
    type Error: Error + Send + Sync + 'static;

    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Exn,
        F: FnOnce() -> A;
}

impl<T, E> ResultExt for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type Success = T;
    type Error = E;

    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Exn,
        F: FnOnce() -> A,
    {
        match self {
            Ok(t) => Ok(t),
            Err(error) => Err(err().with_source(error)),
        }
    }
}

impl<T> ResultExt for Option<T> {
    type Success = T;
    type Error = Infallible;

    fn or_raise<A, F>(self, err: F) -> Result<Self::Success, A>
    where
        A: Exn,
        F: FnOnce() -> A,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(err()),
        }
    }
}

macro_rules! impl_exn {
    ($error_type:ty) => {
        impl Error for $error_type {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                self.source
                    .as_ref()
                    .map(|s| s.as_ref() as &(dyn Error + 'static))
            }
        }

        impl Exn for $error_type {
            fn with_source(mut self, err: impl Error + Send + Sync + 'static) -> Self {
                self.source = Some(Box::new(err));
                self
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use super::*;

    #[derive(Debug, Default)]
    struct TestError {
        source: Option<Box<dyn Error + Send + Sync + 'static>>,
    }
    impl Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "failed successfully")
        }
    }
    impl_exn!(TestError);

    #[test]
    fn walk_source() {
        let source = std::io::Error::from(std::io::ErrorKind::Unsupported);
        let test_error = TestError::default().with_source(source);

        assert!(test_error.source().is_some(), "can get source error back");
    }

    #[test]
    fn exn_from_result() {
        let error = || TestError::default();
        let source: std::io::Result<()> =
            Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
        let result_error = source.or_raise(error);

        assert!(
            result_error.is_err(),
            "can convert Source Result to Result with Exn"
        );
        assert!(
            result_error.unwrap_err().source().is_some(),
            "can get source error back"
        );
    }

    #[test]
    fn exn_from_option() {
        let error = || TestError::default();
        let opt: Option<()> = None;
        let result_error = opt.or_raise(error);

        assert!(result_error.is_err(), "can convert None to Result::Err");
    }
}
