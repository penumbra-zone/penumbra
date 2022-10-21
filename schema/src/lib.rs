// #[macro_use]
// extern crate penumbra_schema_macro;

// use penumbra_schema_core::Key;

use std::{any::TypeId, convert::Infallible, fmt::Display};

pub trait Key {
    type Value;

    fn key(self) -> String;
}

pub trait Collection {
    type Value;

    fn key(self) -> String;
}

pub trait Input<In, T, Err> {
    fn input(input: In) -> Result<T, Err>;
}

pub trait Output<T, Out> {
    fn output(output: T) -> Out;
}

pub struct Builder<In, Out, Err> {
    #[allow(clippy::type_complexity)]
    registry: Vec<(TypeId, fn(In) -> Result<Out, Err>)>,
}

impl<In, Out, Err> Builder<In, Out, Err> {
    pub fn new() -> Self {
        Self {
            registry: Vec::new(),
        }
    }

    pub fn register<I: Input<In, T, Err>, O: Output<T, Out>, T: 'static>(&mut self) {
        const fn converter<I: Input<In, T, Err>, O: Output<T, Out>, In, Out, Err, T>(
        ) -> fn(In) -> Result<Out, Err> {
            fn convert<In, Out, Err, I, O, T>(input: In) -> Result<Out, Err>
            where
                I: Input<In, T, Err>,
                O: Output<T, Out>,
            {
                I::input(input).map(O::output)
            }

            convert::<In, Out, Err, I, O, T>
        }

        self.registry
            .push((TypeId::of::<T>(), converter::<I, O, In, Out, Err, T>()));
    }
}

impl<In, Out, Err> Default for Builder<In, Out, Err> {
    fn default() -> Self {
        Self::new()
    }
}

impl<In, Out, Err> From<Builder<In, Out, Err>> for Converter<In, Out, Err> {
    fn from(mut builder: Builder<In, Out, Err>) -> Self {
        builder.registry.sort_by_key(|(id, _)| *id);
        Self {
            registry: builder.registry,
        }
    }
}

pub struct Converter<In, Out, Err> {
    #[allow(clippy::type_complexity)]
    registry: Vec<(TypeId, fn(In) -> Result<Out, Err>)>,
}

impl<In, Out, Err> Converter<In, Out, Err> {
    pub fn convert<T: 'static>(&self, input: In) -> Option<Result<Out, Err>> {
        self.registry
            .binary_search_by_key(&TypeId::of::<T>(), |(id, _)| *id)
            .map(|index| (self.registry[index].1)(input))
            .ok()
    }
}

// schema! {
//     self as pub mod root;

//     governance {
//         proposal(id: ProposalId) {
//             voting_start: u64;
//         }
//     }
// }

// Generates:

pub mod root {
    pub fn converter<In, I, O, Out, Err>() -> crate::Converter<In, Out, Err>
    where
        I: crate::Input<In, u64, Err>,
        O: crate::Output<u64, Out>,
    {
        let mut builder: crate::Builder<In, Out, Err> = crate::Builder::new();
        builder.register::<I, O, u64>();
        // ...
        builder.into()
    }
}

fn main() {
    struct BytesTruncated;
    struct Displayed;

    impl Input<&[u8], u64, Infallible> for BytesTruncated {
        fn input(input: &[u8]) -> Result<u64, Infallible> {
            let mut bytes = [0u8; 8];
            for (to, from) in bytes.as_mut_slice().iter_mut().zip(input.iter()) {
                *to = *from;
            }
            Ok(u64::from_le_bytes(bytes))
        }
    }

    impl<T: Display> Output<T, String> for Displayed {
        fn output(output: T) -> String {
            format!("{}", output)
        }
    }

    let converter = root::converter::<&[u8], BytesTruncated, Displayed, String, Infallible>();

    let output = converter.convert::<u64>(&[1]);
}

//     pub mod governance {
//         #[derive(Clone)]
//         pub struct Path(pub(super) String);

//         // this is because it was top-level
//         impl Path {
//             pub fn root() -> Path {
//                 Path("governance".to_string())
//             }
//         }

//         impl Path {
//             pub fn proposal(self) -> proposal::Path {
//                 self.0.push_str("/");
//                 self.0.push_str("proposal");
//                 proposal::Path(self.0)
//             }
//         }

//         pub mod proposal {
//             #[derive(Clone)]
//             pub struct Path(pub(super) String);

//             impl Path {
//                 pub fn with(self, id: ProposalId) -> id::Path {
//                     self.0.push_str("/");
//                     self.0.push_str(&id.to_string());
//                     id::Path(self.0)
//                 }
//             }

//             pub mod id {
//                 #[derive(Clone)]
//                 pub struct Path(pub(super) String);

//                 impl Path {
//                     pub fn voting_start(self) -> voting_start::Path {
//                         self.0.push_str("/");
//                         self.0.push_str("voting_start");
//                         voting_start::Path(self.0)
//                     }
//                 }

//                 pub mod voting_start {
//                     #[derive(Clone)]
//                     pub struct Path(pub(super) String);

//                     // These are only here because it's a leaf

//                     impl Key for Path {
//                         type Value = u64;

//                         fn key(self) -> String {
//                             self.0
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// state.get(key!("governance/proposal/{id}/voting_start" in root))

// // generates:

// state.get(root::governance::Path::root().proposal().with(id).voting_start())
