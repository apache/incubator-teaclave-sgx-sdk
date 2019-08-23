//! An ordered list of middleware that performs tested transformations.

use std::prelude::v1::*;
use Error;
use wire::middleware;
use std;

/// A middleware pipeline.
pub trait Pipeline : std::fmt::Debug
{
    fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
    fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error>;
}

/// Creates an instance of the default middleware.
pub fn default() -> Default {
    Default::default()
}

/// Defines a new middleware pipeline that implements `Pipeline`.
#[macro_export]
macro_rules! define_middleware_pipeline {
    ($ty:ident { $( $mw_name:ident : $mw_ty:ty ),+ } ) => {
        #[derive(Clone, Debug)]
        pub struct $ty
        {
            $( pub $mw_name : $mw_ty),+
        }

        impl $ty
        {
            /// Gets the middleware pipeline.
            pub fn middleware_mut(&mut self) -> ::std::collections::VecDeque<&mut dyn $crate::wire::Middleware> {
                let mut middleware = ::std::collections::VecDeque::new();

                $( middleware.push_front(&mut self.$mw_name as &mut dyn $crate::wire::Middleware); )+

                middleware
            }
        }

        impl $crate::wire::middleware::Pipeline for $ty
        {
            fn encode_data(&mut self, mut data: Vec<u8>)
                -> Result<Vec<u8>, $crate::Error> {
                use $crate::wire::Middleware;

                $( data = self.$mw_name.encode_data(data)?; )+

                Ok(data)
            }

            fn decode_data(&mut self, mut data: Vec<u8>)
                -> Result<Vec<u8>, $crate::Error> {
                for middleware in self.middleware_mut() {
                    data = middleware.decode_data(data)?;
                }

                Ok(data)
            }
        }
    }
}

// The default middleware pipeline.
define_middleware_pipeline!(Default {
    compression: middleware::compression::Compression
});

impl std::default::Default for Default
{
    fn default() -> Self {
        Default {
            compression: middleware::compression::Compression::Disabled,
        }
    }
}

#[cfg(test)]
mod test
{
    use Error;
    use wire::middleware::Pipeline;
    use wire;

    define_middleware_pipeline!(NullPipeline {
        encryption: NullMiddleware,
        compression: NullMiddleware
    });

    #[derive(Clone, Debug)]
    pub struct NullMiddleware;

    impl wire::Middleware for NullMiddleware
    {
        fn encode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> { Ok(data) }
        fn decode_data(&mut self, data: Vec<u8>) -> Result<Vec<u8>, Error> { Ok(data) }
    }

    #[test]
    fn successfully_passes_data_through_the_pipeline() {
        let mut null_pipeline = NullPipeline {
            encryption: NullMiddleware,
            compression: NullMiddleware,
        };

        let data = vec![7, 2, 5, 5, 1, 2, 3, 4, 2, 4, 8];

        assert_eq!(null_pipeline.encode_data(data.clone()).unwrap(), data.clone());
        assert_eq!(null_pipeline.decode_data(data.clone()).unwrap(), data.clone());
    }
}

