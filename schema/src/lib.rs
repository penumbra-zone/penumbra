use core::fmt;

use clap::Parser;

#[macro_use]
extern crate clap;

mod clap_extracted;

pub trait DisplayPath {
    fn fmt(&self, separator: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<T: DisplayPath> DisplayPath for &T {
    fn fmt(&self, separator: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, separator, f)
    }
}

impl<T: DisplayPath> DisplayPath for &mut T {
    fn fmt(&self, separator: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, separator, f)
    }
}

// The separator argument must be used to (un)escape separators if they occur in the segment string
pub trait Segment<Schema>: Sized {
    type ParseError;

    fn fmt(&self, separator: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn parse(separator: &str, string: &str) -> Result<Self, Self::ParseError>;
}

pub struct FormatPath<P>(pub &'static str, pub P);

impl<P: DisplayPath> ::core::fmt::Display for FormatPath<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(self.0, f)
    }
}

pub trait Typed {
    type Value;
}

pub trait Homogenize<TryFrom, Error, Into> {
    fn convert(&self, try_from: TryFrom) -> Result<Into, Error>;
}

pub trait Parse<Error>: Sized {
    // TODO: rework this to remove the generic
    fn parse(separator: &str, segments: &[&str]) -> Result<Self, Error>;
}

pub struct InvalidPath {
    pub depth: usize,
}

pub struct ParseError<'s, P> {
    pub prefix: P, // TODO: actually ends up being an OwnedPath of some kind
    pub remainder: &'s [&'s str],
    pub error: ParseErrorKind,
}

pub enum ParseErrorKind {
    InvalidSegment(Box<dyn ::std::error::Error>),
    WrongLength { expected: usize, actual: usize },
}

// TODO: parsing should build a path, then convert it to a key

// An example:

// pub fn getter<'de, 'key, P, K: Typed>(
//     key: K,
// ) -> (String, fn(&'de [u8]) -> Result<K::Value, anyhow::Error>)
// where
//     P: prost::Message + Default + From<<K as Typed>::Value>,
//     K::Value: penumbra_proto::Protobuf<P>,
//     <K::Value as TryFrom<P>>::Error: Into<anyhow::Error>,
//     schema::Key<'key>: From<K>,
// {
//     (
//         format!("{}", FormatPath("/", schema::Key::from(key))),
//         <K::Value as penumbra_proto::Protobuf<P>>::decode,
//     )
// }

// pub fn putter<'key, P, K: Typed>(key: K, value: &K::Value) -> (String, Vec<u8>)
// where
//     P: prost::Message + Default + From<<K as Typed>::Value>,
//     K::Value: penumbra_proto::Protobuf<P>,
//     <K::Value as TryFrom<P>>::Error: Into<anyhow::Error>,
//     schema::Key<'key>: From<K>,
// {
//     (
//         format!("{}", FormatPath("/", schema::Key::from(key))),
//         penumbra_proto::Protobuf::encode_to_vec(value),
//     )
// }

fn main() {
    // let (path, decode) = getter(schema::governance().proposal().id(&5).voting_start());
    // assert_eq!(path, "governance/proposal/5/voting_start");
    #[derive(Clone, Parser)]
    struct Opts {
        #[clap(subcommand)]
        query: Query,
    }

    #[derive(Clone, Subcommand)]
    enum Query {
        Key(schema::OwnedKey),
    }

    let opts = Opts::parse();

    match opts.query {
        Query::Key(key) => println!("{}", FormatPath("/", key)),
    }
}

// This will need to be done for all used types -- a quick macro to make it easy?
impl Segment<schema::Schema> for u64 {
    type ParseError = <u64 as core::str::FromStr>::Err;

    fn fmt(&self, _separator: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }

    fn parse(_separator: &str, string: &str) -> Result<Self, Self::ParseError> {
        string.parse()
    }
}

// pub mod schema {
//     schema! {
//          governance {
//             proposal(id: u64) {
//                 voting_start: u64;
//             }
//         }
//     }
// }

// Generates:

pub mod schema {
    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    pub struct Schema;

    impl From<&Schema> for Schema {
        fn from(_: &Schema) -> Self {
            Schema
        }
    }

    impl Schema {
        pub fn root<'a>() -> Path<'a> {
            Path {
                params: Params {
                    __: ::core::marker::PhantomData,
                },
                parent: Schema,
            }
        }

        pub fn owned_root() -> OwnedPath {
            OwnedPath {
                params: OwnedParams {},
                parent: Schema,
            }
        }
    }

    // Only for the root of the schema, generate these:
    pub fn governance<'a>() -> governance::Path<'a> {
        Schema::root().governance()
    }

    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    pub struct Path<'a> {
        params: Params<'a>,
        parent: Schema, // special when root of schema
    }

    // Prefix, Key, OwnedPrefix, and OwnedKey are only pub in the root of the schema

    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    pub struct Prefix<'a> {
        params: Params<'a>,
        child: ::core::option::Option<SubPrefix<'a>>,
    }

    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    pub struct Key<'a> {
        params: Params<'a>,
        child: SubKey<'a>,
    }

    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
    pub struct OwnedPath {
        params: OwnedParams,
        parent: Schema, // special when root of schema
    }

    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
    pub struct OwnedPrefix {
        params: OwnedParams,
        child: ::core::option::Option<OwnedSubPrefix>,
    }

    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args)]
    #[group(skip)]
    pub struct OwnedKey {
        #[clap(flatten)]
        params: OwnedParams,
        #[clap(subcommand)]
        child: OwnedSubKey,
    }

    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    struct Params<'a> {
        __: ::core::marker::PhantomData<&'a ()>,
    }

    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args)]
    #[group(skip)]
    struct OwnedParams {}

    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    enum SubPrefix<'a> {
        governance(governance::Prefix<'a>),
    }

    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
    enum OwnedSubPrefix {
        governance(governance::OwnedPrefix),
    }

    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    #[derive(
        ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
    )]
    enum SubKey<'a> {
        governance(governance::Key<'a>),
    }

    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Subcommand)]
    enum OwnedSubKey {
        governance(governance::OwnedKey),
    }

    impl<Error> crate::Parse<Error> for OwnedKey
    where
        Error: ::core::convert::From<<u64 as crate::Segment<Schema>>::ParseError>
            + ::core::convert::From<crate::InvalidPath>,
    {
        fn parse(separator: &str, segments: &[&str]) -> Result<Self, Error> {
            match segments {
                ["governance", rest @ ..] => Ok(OwnedKey {
                    params: OwnedParams {},
                    child: OwnedSubKey::governance(<governance::OwnedKey as crate::Parse<
                        Error,
                    >>::parse(separator, rest)?),
                }),
                _ => Err(crate::InvalidPath { depth: 0 }.into()),
            }
        }
    }

    impl<'a> From<&'a OwnedSubPrefix> for SubPrefix<'a> {
        fn from(prefix: &'a OwnedSubPrefix) -> Self {
            match prefix {
                OwnedSubPrefix::governance(prefix) => SubPrefix::governance(prefix.into()),
            }
        }
    }

    impl<'a> From<SubPrefix<'a>> for OwnedSubPrefix {
        fn from(prefix: SubPrefix<'a>) -> Self {
            match prefix {
                SubPrefix::governance(prefix) => OwnedSubPrefix::governance(prefix.into()),
            }
        }
    }

    impl<'a> From<&'a OwnedSubKey> for SubKey<'a> {
        fn from(key: &'a OwnedSubKey) -> Self {
            match key {
                OwnedSubKey::governance(key) => SubKey::governance(key.into()),
            }
        }
    }

    impl<'a> From<SubKey<'a>> for OwnedSubKey {
        fn from(key: SubKey<'a>) -> Self {
            match key {
                SubKey::governance(key) => OwnedSubKey::governance(key.into()),
            }
        }
    }

    impl<'a> crate::DisplayPath for Key<'a> {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let Params { .. } = &self.params;
            // special: don't print anything, because we're at the root of the schema
            match &self.child {
                SubKey::governance(child) => {
                    child.fmt(separator, f)?;
                }
            }
            Ok(())
        }
    }

    impl<'a> crate::DisplayPath for Prefix<'a> {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let Params { .. } = &self.params;
            // special: don't print anything, because we're at the root of the schema
            match &self.child {
                ::core::option::Option::Some(SubPrefix::governance(child)) => {
                    child.fmt(separator, f)?;
                }
                ::core::option::Option::None => {
                    write!(f, "{}", separator)?;
                }
            }
            Ok(())
        }
    }

    impl crate::DisplayPath for OwnedKey {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let OwnedParams { .. } = &self.params;
            // special: don't print anything, because we're at the root of the schema
            match &self.child {
                OwnedSubKey::governance(child) => {
                    child.fmt(separator, f)?;
                }
            }
            Ok(())
        }
    }

    impl crate::DisplayPath for OwnedPrefix {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let OwnedParams { .. } = &self.params;
            // special: don't print anything, because we're at the root of the schema
            match &self.child {
                ::core::option::Option::Some(OwnedSubPrefix::governance(child)) => {
                    child.fmt(separator, f)?;
                }
                ::core::option::Option::None => {
                    write!(f, "{}", separator)?;
                }
            }
            Ok(())
        }
    }

    impl<'a> crate::DisplayPath for Path<'a> {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let key: Prefix<'a> = (*self).into();
            key.fmt(separator, f)
        }
    }

    impl crate::DisplayPath for OwnedPath {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            let path: Path<'_> = self.into();
            let key: Prefix<'_> = path.into();
            key.fmt(separator, f)
        }
    }

    impl<'a, TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for Key<'a>
    where
        governance::Key<'a>: crate::Homogenize<TryFrom, Error, Into>,
    {
        fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
            match self.child {
                SubKey::governance(ref key) => crate::Homogenize::convert(key, try_from),
            }
        }
    }

    impl<TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for OwnedKey
    where
        governance::OwnedKey: crate::Homogenize<TryFrom, Error, Into>,
    {
        fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
            match self.child {
                OwnedSubKey::governance(ref key) => crate::Homogenize::convert(key, try_from),
            }
        }
    }

    impl<'a> From<&'a OwnedParams> for Params<'a> {
        fn from(params: &'a OwnedParams) -> Self {
            let OwnedParams { .. } = params;
            Params {
                __: ::core::marker::PhantomData,
            }
        }
    }

    impl<'a> From<Params<'a>> for OwnedParams {
        fn from(params: Params<'a>) -> Self {
            let Params { .. } = params;
            OwnedParams {}
        }
    }

    impl<'a> From<&'a OwnedPath> for Path<'a> {
        fn from(root: &'a OwnedPath) -> Self {
            let OwnedPath { parent, params } = root;
            let parent = parent.into();
            let params = params.into();
            Path { parent, params }
        }
    }

    impl<'a> From<Path<'a>> for OwnedPath {
        fn from(root: Path<'a>) -> Self {
            let Path { parent, params } = root;
            let parent = parent.into();
            let params = params.into();
            OwnedPath { parent, params }
        }
    }

    impl<'a> From<&'a OwnedKey> for Key<'a> {
        fn from(key: &'a OwnedKey) -> Self {
            let OwnedKey { params, child } = key;
            let params = params.into();
            let child = child.into();
            Key { params, child }
        }
    }

    impl<'a> From<Key<'a>> for OwnedKey {
        fn from(key: Key<'a>) -> Self {
            let Key { params, child } = key;
            let params = params.into();
            let child = child.into();
            OwnedKey { params, child }
        }
    }

    impl<'a> From<&'a OwnedPrefix> for Prefix<'a> {
        fn from(key: &'a OwnedPrefix) -> Self {
            let OwnedPrefix { params, child } = key;
            let params = params.into();
            let child = child.as_ref().map(::core::convert::Into::into);
            Prefix { params, child }
        }
    }

    impl<'a> From<Prefix<'a>> for OwnedPrefix {
        fn from(key: Prefix<'a>) -> Self {
            let Prefix { params, child } = key;
            let params = params.into();
            let child = child.map(::core::convert::Into::into);
            OwnedPrefix { params, child }
        }
    }

    impl<'a> From<Path<'a>> for Prefix<'a> {
        fn from(root: Path<'a>) -> Self {
            let Path {
                parent: Schema,
                params,
            } = root;
            let key = Prefix {
                params,
                child: None,
            };

            key
        }
    }

    impl From<OwnedPath> for OwnedPrefix {
        fn from(root: OwnedPath) -> Self {
            let OwnedPath {
                parent: root,
                params,
            } = root;
            let key = OwnedPrefix {
                params,
                child: None,
            };

            key
        }
    }

    pub mod governance {
        #[derive(
            ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
        )]
        pub struct Path<'a> {
            params: Params<'a>,
            parent: super::Path<'a>,
        }

        #[derive(
            ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
        )]
        pub(super) struct Prefix<'a> {
            params: Params<'a>,
            child: ::core::option::Option<SubPrefix<'a>>,
        }

        #[derive(
            ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
        )]
        pub(super) struct Key<'a> {
            params: Params<'a>,
            child: SubKey<'a>,
        }

        #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        pub struct OwnedPath {
            params: OwnedParams,
            parent: super::OwnedPath,
        }

        #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        pub(super) struct OwnedPrefix {
            params: OwnedParams,
            child: ::core::option::Option<OwnedSubPrefix>,
        }

        #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args)]
        #[group(skip)]
        pub(super) struct OwnedKey {
            #[clap(flatten)]
            params: OwnedParams,
            #[clap(subcommand)]
            child: OwnedSubKey,
        }

        #[derive(
            ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
        )]
        struct Params<'a> {
            __: ::core::marker::PhantomData<&'a ()>,
        }

        #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args)]
        #[group(skip)]
        struct OwnedParams {}

        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        #[derive(
            ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
        )]
        enum SubPrefix<'a> {
            proposal(proposal::Prefix<'a>),
        }

        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
        enum OwnedSubPrefix {
            proposal(proposal::OwnedPrefix),
        }

        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        #[derive(
            ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
        )]
        enum SubKey<'a> {
            proposal(proposal::Key<'a>),
        }

        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        #[derive(
            ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Subcommand,
        )]
        enum OwnedSubKey {
            proposal(proposal::OwnedKey),
        }

        impl<Error> crate::Parse<Error> for OwnedKey
        where
            Error: ::core::convert::From<<u64 as crate::Segment<super::Schema>>::ParseError>
                + ::core::convert::From<crate::InvalidPath>,
        {
            fn parse(separator: &str, segments: &[&str]) -> Result<Self, Error> {
                match segments {
                    ["proposal", rest @ ..] => Ok(OwnedKey {
                        params: OwnedParams {},
                        child: OwnedSubKey::proposal(<proposal::OwnedKey as crate::Parse<
                            Error,
                        >>::parse(
                            separator, rest
                        )?),
                    }),
                    _ => Err(crate::InvalidPath { depth: 1 }.into()),
                }
            }
        }

        impl<'a> From<&'a OwnedSubPrefix> for SubPrefix<'a> {
            fn from(prefix: &'a OwnedSubPrefix) -> Self {
                match prefix {
                    OwnedSubPrefix::proposal(prefix) => SubPrefix::proposal(prefix.into()),
                }
            }
        }

        impl<'a> From<SubPrefix<'a>> for OwnedSubPrefix {
            fn from(prefix: SubPrefix<'a>) -> Self {
                match prefix {
                    SubPrefix::proposal(prefix) => OwnedSubPrefix::proposal(prefix.into()),
                }
            }
        }

        impl<'a> From<&'a OwnedSubKey> for SubKey<'a> {
            fn from(key: &'a OwnedSubKey) -> Self {
                match key {
                    OwnedSubKey::proposal(key) => SubKey::proposal(key.into()),
                }
            }
        }

        impl<'a> From<SubKey<'a>> for OwnedSubKey {
            fn from(key: SubKey<'a>) -> Self {
                match key {
                    SubKey::proposal(key) => OwnedSubKey::proposal(key.into()),
                }
            }
        }

        impl<'a> super::Path<'a> {
            pub fn governance(self) -> Path<'a> {
                Path {
                    parent: self,
                    params: Params {
                        __: ::core::marker::PhantomData,
                    },
                }
            }
        }

        impl super::OwnedPath {
            pub fn governance(self) -> OwnedPath {
                OwnedPath {
                    parent: self,
                    params: OwnedParams {},
                }
            }
        }

        impl<'a> crate::DisplayPath for Key<'a> {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                let Params { .. } = &self.params;
                write!(f, "governance")?;
                write!(f, "{}", separator)?;
                match &self.child {
                    SubKey::proposal(child) => {
                        child.fmt(separator, f)?;
                    }
                }
                Ok(())
            }
        }

        impl<'a> crate::DisplayPath for Prefix<'a> {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                let Params { .. } = &self.params;
                write!(f, "governance")?;
                write!(f, "{}", separator)?;
                match &self.child {
                    ::core::option::Option::Some(SubPrefix::proposal(child)) => {
                        child.fmt(separator, f)?;
                    }
                    ::core::option::Option::None => {}
                }
                Ok(())
            }
        }

        impl crate::DisplayPath for OwnedKey {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                let OwnedParams { .. } = &self.params;
                write!(f, "governance")?;
                write!(f, "{}", separator)?;
                match &self.child {
                    OwnedSubKey::proposal(child) => {
                        child.fmt(separator, f)?;
                    }
                }
                Ok(())
            }
        }

        impl crate::DisplayPath for OwnedPrefix {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                let OwnedParams { .. } = &self.params;
                write!(f, "governance")?;
                write!(f, "{}", separator)?;
                match &self.child {
                    ::core::option::Option::Some(OwnedSubPrefix::proposal(child)) => {
                        child.fmt(separator, f)?;
                    }
                    ::core::option::Option::None => {}
                }
                Ok(())
            }
        }

        impl<'a> crate::DisplayPath for Path<'a> {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                let key: super::Prefix<'a> = (*self).into();
                key.fmt(separator, f)
            }
        }

        impl crate::DisplayPath for OwnedPath {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                let path: Path<'_> = self.into();
                let key: super::Prefix<'_> = path.into();
                key.fmt(separator, f)
            }
        }

        impl<'a, TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for Key<'a>
        where
            proposal::Key<'a>: crate::Homogenize<TryFrom, Error, Into>,
        {
            fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                match self.child {
                    SubKey::proposal(ref key) => crate::Homogenize::convert(key, try_from),
                }
            }
        }

        impl<TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for OwnedKey
        where
            proposal::OwnedKey: crate::Homogenize<TryFrom, Error, Into>,
        {
            fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                match self.child {
                    OwnedSubKey::proposal(ref key) => crate::Homogenize::convert(key, try_from),
                }
            }
        }

        impl<'a> From<&'a OwnedParams> for Params<'a> {
            fn from(params: &'a OwnedParams) -> Self {
                let OwnedParams { .. } = params;
                Params {
                    __: ::core::marker::PhantomData,
                }
            }
        }

        impl<'a> From<Params<'a>> for OwnedParams {
            fn from(params: Params<'a>) -> Self {
                let Params { .. } = params;
                OwnedParams {}
            }
        }

        impl<'a> From<&'a OwnedPath> for Path<'a> {
            fn from(root: &'a OwnedPath) -> Self {
                let OwnedPath { parent, params } = root;
                let parent = parent.into();
                let params = params.into();
                Path { parent, params }
            }
        }

        impl<'a> From<Path<'a>> for OwnedPath {
            fn from(root: Path<'a>) -> Self {
                let Path { parent, params } = root;
                let parent = parent.into();
                let params = params.into();
                OwnedPath { parent, params }
            }
        }

        impl<'a> From<&'a OwnedKey> for Key<'a> {
            fn from(key: &'a OwnedKey) -> Self {
                let OwnedKey { params, child } = key;
                let params = params.into();
                let child = child.into();
                Key { params, child }
            }
        }

        impl<'a> From<Key<'a>> for OwnedKey {
            fn from(key: Key<'a>) -> Self {
                let Key { params, child } = key;
                let params = params.into();
                let child = child.into();
                OwnedKey { params, child }
            }
        }

        impl<'a> From<&'a OwnedPrefix> for Prefix<'a> {
            fn from(key: &'a OwnedPrefix) -> Self {
                let OwnedPrefix { params, child } = key;
                let params = params.into();
                let child = child.as_ref().map(::core::convert::Into::into);
                Prefix { params, child }
            }
        }

        impl<'a> From<Prefix<'a>> for OwnedPrefix {
            fn from(key: Prefix<'a>) -> Self {
                let Prefix { params, child } = key;
                let params = params.into();
                let child = child.map(::core::convert::Into::into);
                OwnedPrefix { params, child }
            }
        }

        impl<'a> From<Path<'a>> for super::Prefix<'a> {
            fn from(root: Path<'a>) -> Self {
                let Path {
                    parent: root,
                    params,
                } = root;
                let key = Prefix {
                    params,
                    child: None,
                };

                let super::Path {
                    parent: super::Schema,
                    params,
                } = root;
                let key = super::Prefix {
                    params,
                    child: Some(super::SubPrefix::governance(key)),
                };

                key
            }
        }

        impl From<OwnedPath> for super::OwnedPrefix {
            fn from(root: OwnedPath) -> Self {
                let OwnedPath {
                    parent: root,
                    params,
                } = root;
                let key = OwnedPrefix {
                    params,
                    child: None,
                };

                let super::OwnedPath {
                    params,
                    parent: super::Schema,
                } = root;
                let key = super::OwnedPrefix {
                    params,
                    child: Some(super::OwnedSubPrefix::governance(key)),
                };

                key
            }
        }

        pub mod proposal {
            #[derive(
                ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
            )]
            pub struct Path<'a> {
                params: Params<'a>,
                parent: super::Path<'a>,
            }

            #[derive(
                ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
            )]
            pub(super) struct Prefix<'a> {
                params: Params<'a>,
                child: ::core::option::Option<SubPrefix<'a>>,
            }

            #[derive(
                ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
            )]
            pub(super) struct Key<'a> {
                params: Params<'a>,
                child: SubKey<'a>,
            }

            #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
            pub struct OwnedPath {
                params: OwnedParams,
                parent: super::OwnedPath,
            }

            #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
            pub(super) struct OwnedPrefix {
                params: OwnedParams,
                child: ::core::option::Option<OwnedSubPrefix>,
            }

            #[derive(
                ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args,
            )]
            #[group(skip)]
            pub(super) struct OwnedKey {
                #[clap(flatten)]
                params: OwnedParams,
                #[clap(flatten)] // special: child has args
                child: OwnedSubKey,
            }

            #[derive(
                ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
            )]
            struct Params<'a> {
                __: ::core::marker::PhantomData<&'a ()>,
            }

            #[derive(
                ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args,
            )]
            #[group(skip)]
            struct OwnedParams {}

            #[allow(non_camel_case_types)]
            #[non_exhaustive]
            #[derive(
                ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
            )]
            enum SubPrefix<'a> {
                id(id::Prefix<'a>),
            }

            #[allow(non_camel_case_types)]
            #[non_exhaustive]
            #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
            enum OwnedSubPrefix {
                id(id::OwnedPrefix),
            }

            #[allow(non_camel_case_types)]
            #[non_exhaustive]
            #[derive(
                ::core::clone::Clone, ::core::marker::Copy, ::core::cmp::PartialEq, ::core::cmp::Eq,
            )]
            enum SubKey<'a> {
                id(id::Key<'a>),
            }

            #[allow(non_camel_case_types)]
            #[non_exhaustive]
            #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
            enum OwnedSubKey {
                id(id::OwnedKey),
            }

            impl<Error> crate::Parse<Error> for OwnedKey
            where
                Error: ::core::convert::From<<u64 as crate::Segment<super::super::Schema>>::ParseError>
                    + ::core::convert::From<crate::InvalidPath>,
            {
                fn parse(separator: &str, segments: &[&str]) -> Result<Self, Error> {
                    // special: when child has args, forward directly
                    Ok(OwnedKey {
                        params: OwnedParams {},
                        child: OwnedSubKey::id(<id::OwnedKey as crate::Parse<Error>>::parse(
                            separator, segments,
                        )?),
                    })
                }
            }

            // Child has args, so we have to do this manually, because we can't be a struct
            impl ::clap::FromArgMatches for OwnedSubKey {
                fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
                    id::OwnedKey::from_arg_matches(matches).map(OwnedSubKey::id)
                }

                fn update_from_arg_matches(
                    &mut self,
                    matches: &clap::ArgMatches,
                ) -> Result<(), clap::Error> {
                    match self {
                        OwnedSubKey::id(ref mut key) => key.update_from_arg_matches(matches),
                    }
                }
            }

            impl ::clap::Args for OwnedSubKey {
                fn augment_args(cmd: clap::Command) -> clap::Command {
                    id::OwnedKey::augment_args(cmd)
                }

                fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
                    id::OwnedKey::augment_args_for_update(cmd)
                }
            }

            impl<'a> From<&'a OwnedSubPrefix> for SubPrefix<'a> {
                fn from(prefix: &'a OwnedSubPrefix) -> Self {
                    match prefix {
                        OwnedSubPrefix::id(prefix) => SubPrefix::id(prefix.into()),
                    }
                }
            }

            impl<'a> From<SubPrefix<'a>> for OwnedSubPrefix {
                fn from(prefix: SubPrefix<'a>) -> Self {
                    match prefix {
                        SubPrefix::id(prefix) => OwnedSubPrefix::id(prefix.into()),
                    }
                }
            }

            impl<'a> From<&'a OwnedSubKey> for SubKey<'a> {
                fn from(key: &'a OwnedSubKey) -> Self {
                    match key {
                        OwnedSubKey::id(key) => SubKey::id(key.into()),
                    }
                }
            }

            impl<'a> From<SubKey<'a>> for OwnedSubKey {
                fn from(key: SubKey<'a>) -> Self {
                    match key {
                        SubKey::id(key) => OwnedSubKey::id(key.into()),
                    }
                }
            }

            impl<'a> super::Path<'a> {
                pub fn proposal(self) -> Path<'a> {
                    Path {
                        parent: self,
                        params: Params {
                            __: ::core::marker::PhantomData,
                        },
                    }
                }
            }

            impl super::OwnedPath {
                pub fn proposal(self) -> OwnedPath {
                    OwnedPath {
                        parent: self,
                        params: OwnedParams {},
                    }
                }
            }

            impl<'a> crate::DisplayPath for Key<'a> {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    let Params { .. } = &self.params;
                    write!(f, "proposal")?;
                    write!(f, "{}", separator)?;
                    match &self.child {
                        SubKey::id(child) => child.fmt(separator, f)?,
                    };
                    Ok(())
                }
            }

            impl<'a> crate::DisplayPath for Prefix<'a> {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    let Params { .. } = &self.params;
                    write!(f, "proposal")?;
                    write!(f, "{}", separator)?;
                    match &self.child {
                        ::core::option::Option::Some(SubPrefix::id(prefix)) => {
                            prefix.fmt(separator, f)?;
                        }
                        ::core::option::Option::None => {
                            panic!()
                        }
                    }
                    Ok(())
                }
            }

            impl crate::DisplayPath for OwnedKey {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    let OwnedParams { .. } = &self.params;
                    write!(f, "proposal")?;
                    write!(f, "{}", separator)?;
                    match &self.child {
                        OwnedSubKey::id(child) => {
                            child.fmt(separator, f)?;
                        }
                    }
                    Ok(())
                }
            }

            impl crate::DisplayPath for OwnedPrefix {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    let OwnedParams { .. } = &self.params;
                    write!(f, "proposal")?;
                    write!(f, "{}", separator)?;
                    match &self.child {
                        ::core::option::Option::Some(OwnedSubPrefix::id(child)) => {
                            child.fmt(separator, f)?;
                        }
                        ::core::option::Option::None => {}
                    }
                    Ok(())
                }
            }

            impl<'a> crate::DisplayPath for Path<'a> {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    let key: super::super::Prefix<'a> = (*self).into();
                    key.fmt(separator, f)
                }
            }

            impl crate::DisplayPath for OwnedPath {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    let path: Path<'_> = self.into();
                    let key: super::super::Prefix<'_> = path.into();
                    key.fmt(separator, f)
                }
            }

            impl<'a, TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for Key<'a>
            where
                id::Key<'a>: crate::Homogenize<TryFrom, Error, Into>,
            {
                fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                    match &self.child {
                        SubKey::id(child) => child.convert(try_from),
                    }
                }
            }

            impl<TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for OwnedKey
            where
                id::OwnedKey: crate::Homogenize<TryFrom, Error, Into>,
            {
                fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                    match self.child {
                        OwnedSubKey::id(ref key) => crate::Homogenize::convert(key, try_from),
                    }
                }
            }

            impl<'a> From<&'a OwnedParams> for Params<'a> {
                fn from(params: &'a OwnedParams) -> Self {
                    let OwnedParams { .. } = params;
                    Params {
                        __: ::core::marker::PhantomData,
                    }
                }
            }

            impl<'a> From<Params<'a>> for OwnedParams {
                fn from(params: Params<'a>) -> Self {
                    let Params { .. } = params;
                    OwnedParams {}
                }
            }

            impl<'a> From<&'a OwnedPath> for Path<'a> {
                fn from(root: &'a OwnedPath) -> Self {
                    let OwnedPath { parent, params } = root;
                    let parent = parent.into();
                    let params = params.into();
                    Path { parent, params }
                }
            }

            impl<'a> From<Path<'a>> for OwnedPath {
                fn from(root: Path<'a>) -> Self {
                    let Path { parent, params } = root;
                    let parent = parent.into();
                    let params = params.into();
                    OwnedPath { parent, params }
                }
            }

            impl<'a> From<&'a OwnedKey> for Key<'a> {
                fn from(key: &'a OwnedKey) -> Self {
                    let OwnedKey { params, child } = key;
                    let params = params.into();
                    let child = child.into();
                    Key { params, child }
                }
            }

            impl<'a> From<Key<'a>> for OwnedKey {
                fn from(key: Key<'a>) -> Self {
                    let Key { params, child } = key;
                    let params = params.into();
                    let child = child.into();
                    OwnedKey { params, child }
                }
            }

            impl<'a> From<&'a OwnedPrefix> for Prefix<'a> {
                fn from(key: &'a OwnedPrefix) -> Self {
                    let OwnedPrefix { params, child } = key;
                    let params = params.into();
                    let child = child.as_ref().map(::core::convert::Into::into);
                    Prefix { params, child }
                }
            }

            impl<'a> From<Prefix<'a>> for OwnedPrefix {
                fn from(key: Prefix<'a>) -> Self {
                    let Prefix { params, child } = key;
                    let params = params.into();
                    let child = child.map(::core::convert::Into::into);
                    OwnedPrefix { params, child }
                }
            }

            impl<'a> From<Path<'a>> for super::super::Prefix<'a> {
                fn from(root: Path<'a>) -> Self {
                    let Path {
                        parent: root,
                        params,
                    } = root;
                    let key = Prefix {
                        params,
                        child: None,
                    };

                    let super::Path {
                        parent: root,
                        params,
                    } = root;
                    let key = super::Prefix {
                        params,
                        child: Some(super::SubPrefix::proposal(key)),
                    };

                    let super::super::Path {
                        parent: super::super::Schema,
                        params,
                    } = root;
                    let key = super::super::Prefix {
                        params,
                        child: Some(super::super::SubPrefix::governance(key)),
                    };

                    key
                }
            }

            impl From<OwnedPath> for super::super::OwnedPrefix {
                fn from(root: OwnedPath) -> Self {
                    let OwnedPath {
                        parent: root,
                        params,
                    } = root;
                    let key = OwnedPrefix {
                        params,
                        child: None,
                    };

                    let super::OwnedPath {
                        parent: root,
                        params,
                    } = root;
                    let key = super::OwnedPrefix {
                        params,
                        child: Some(super::OwnedSubPrefix::proposal(key)),
                    };

                    let super::super::OwnedPath {
                        params,
                        parent: super::super::Schema,
                    } = root;
                    let key = super::super::OwnedPrefix {
                        params,
                        child: Some(super::super::OwnedSubPrefix::governance(key)),
                    };

                    key
                }
            }

            pub mod id {
                #[derive(
                    ::core::clone::Clone,
                    ::core::marker::Copy,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                pub struct Path<'a> {
                    params: Params<'a>,
                    parent: super::Path<'a>,
                }

                #[derive(
                    ::core::clone::Clone,
                    ::core::marker::Copy,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                pub(super) struct Prefix<'a> {
                    params: Params<'a>,
                    child: ::core::option::Option<SubPrefix<'a>>,
                }

                #[derive(
                    ::core::clone::Clone,
                    ::core::marker::Copy,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                pub(super) struct Key<'a> {
                    params: Params<'a>,
                    child: SubKey<'a>,
                }

                #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
                pub struct OwnedPath {
                    params: OwnedParams,
                    parent: super::OwnedPath,
                }

                #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
                pub(super) struct OwnedPrefix {
                    params: OwnedParams,
                    child: Option<OwnedSubPrefix>,
                }

                #[derive(
                    ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args,
                )]
                #[group(skip)]
                pub(super) struct OwnedKey {
                    #[clap(flatten)]
                    params: OwnedParams,
                    #[clap(subcommand)]
                    child: OwnedSubKey,
                }

                #[derive(
                    ::core::clone::Clone,
                    ::core::marker::Copy,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                struct Params<'a> {
                    id: &'a u64,
                }

                #[derive(
                    ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args,
                )]
                #[group(skip)]
                struct OwnedParams {
                    // #[clap(long)] // only insert if more than one field
                    id: u64,
                }

                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #[derive(
                    ::core::clone::Clone,
                    ::core::marker::Copy,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                enum SubPrefix<'a> {
                    // When there are no sub-prefixes, we need this variant to allow lifetime to exist
                    #[doc(hidden)]
                    __(::core::marker::PhantomData<&'a ()>),
                }

                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #[derive(
                    ::core::clone::Clone,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                    ::clap::Subcommand,
                )]
                enum OwnedSubPrefix {}

                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #[derive(
                    ::core::clone::Clone,
                    ::core::marker::Copy,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                )]
                enum SubKey<'a> {
                    voting_start(voting_start::Key<'a>),
                }

                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #[derive(
                    ::core::clone::Clone,
                    ::core::cmp::PartialEq,
                    ::core::cmp::Eq,
                    ::clap::Subcommand,
                )]
                enum OwnedSubKey {
                    voting_start(voting_start::OwnedKey),
                }

                impl<Error> crate::Parse<Error> for OwnedKey
                where
                    u64: crate::Segment<super::super::super::Schema>,
                    Error: ::core::convert::From<
                            <u64 as crate::Segment<super::super::super::Schema>>::ParseError,
                        > + ::core::convert::From<crate::InvalidPath>,
                {
                    fn parse(separator: &str, segments: &[&str]) -> Result<Self, Error> {
                        match segments {
                            [id, rest @ ..] => {
                                let id =
                                    crate::Segment::parse(id, separator).map_err(Error::from)?;
                                let params = OwnedParams { id };
                                match rest {
                                    ["voting_start", rest @ ..] => Ok(OwnedKey {
                                        params,
                                        child: OwnedSubKey::voting_start(
                                            <voting_start::OwnedKey as crate::Parse<Error>>::parse(
                                                separator, rest,
                                            )?,
                                        ),
                                    }),
                                    _ => Err(crate::InvalidPath { depth: 2 }.into()),
                                }
                            }
                            [] => todo!(),
                        }
                    }
                }

                impl<'a> From<&'a OwnedSubPrefix> for SubPrefix<'a> {
                    fn from(prefix: &'a OwnedSubPrefix) -> Self {
                        match *prefix {} // special case when no prefixes, we need to dereference to prove match is complete
                    }
                }

                impl<'a> From<SubPrefix<'a>> for OwnedSubPrefix {
                    fn from(prefix: SubPrefix<'a>) -> Self {
                        match prefix {
                            SubPrefix::__(_) => unreachable!(), // special case when no prefixes, we need to dereference to prove match is complete
                        }
                    }
                }

                impl<'a> From<&'a OwnedSubKey> for SubKey<'a> {
                    fn from(key: &'a OwnedSubKey) -> Self {
                        match key {
                            OwnedSubKey::voting_start(key) => SubKey::voting_start(key.into()),
                        }
                    }
                }

                impl<'a> From<SubKey<'a>> for OwnedSubKey {
                    fn from(key: SubKey<'a>) -> Self {
                        match key {
                            SubKey::voting_start(key) => OwnedSubKey::voting_start(key.into()),
                        }
                    }
                }

                impl<'a> super::Path<'a> {
                    pub fn id(self, id: &'a u64) -> Path<'a> {
                        Path {
                            parent: self,
                            params: Params { id },
                        }
                    }
                }

                impl super::OwnedPath {
                    pub fn id(self, id: u64) -> OwnedPath {
                        OwnedPath {
                            parent: self,
                            params: OwnedParams { id },
                        }
                    }
                }

                impl<'a> crate::DisplayPath for Key<'a> {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        let Params { id, .. } = &self.params;
                        <u64 as crate::Segment<super::super::super::Schema>>::fmt(
                            id, separator, f,
                        )?;
                        write!(f, "{}", separator)?;
                        match &self.child {
                            SubKey::voting_start(child) => {
                                child.fmt(separator, f)?;
                            }
                        }
                        Ok(())
                    }
                }

                impl<'a> crate::DisplayPath for Prefix<'a> {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        let Params { id, .. } = &self.params;
                        <u64 as crate::Segment<super::super::super::Schema>>::fmt(
                            id, separator, f,
                        )?;
                        write!(f, "{}", separator)?;
                        match &self.child {
                            // special: there is no sub-prefix
                            ::core::option::Option::Some(SubPrefix::__(
                                ::core::marker::PhantomData,
                            )) => {
                                unreachable!()
                            }
                            ::core::option::Option::None => {}
                        }
                        Ok(())
                    }
                }

                impl crate::DisplayPath for OwnedKey {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        let OwnedParams { id, .. } = &self.params;
                        <u64 as crate::Segment<super::super::super::Schema>>::fmt(
                            id, separator, f,
                        )?;
                        write!(f, "{}", separator)?;
                        match &self.child {
                            OwnedSubKey::voting_start(child) => {
                                child.fmt(separator, f)?;
                            }
                        }
                        Ok(())
                    }
                }

                impl crate::DisplayPath for OwnedPrefix {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        let OwnedParams { id, .. } = &self.params;
                        <u64 as crate::Segment<super::super::super::Schema>>::fmt(
                            id, separator, f,
                        )?;
                        write!(f, "{}", separator)?;
                        match &self.child {
                            // special: there is no sub-prefix
                            ::core::option::Option::Some(prefix) => match *prefix {}, // prove it's empty
                            ::core::option::Option::None => {}
                        }
                        Ok(())
                    }
                }

                impl<'a> crate::DisplayPath for Path<'a> {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        let key: super::super::super::Prefix<'a> = (*self).into();
                        key.fmt(separator, f)
                    }
                }

                impl crate::DisplayPath for OwnedPath {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        let path: Path<'_> = self.into();
                        let key: super::super::super::Prefix<'_> = path.into();
                        key.fmt(separator, f)
                    }
                }

                impl<'a, TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for Key<'a>
                where
                    voting_start::Key<'a>: crate::Homogenize<TryFrom, Error, Into>,
                {
                    fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                        match self.child {
                            SubKey::voting_start(ref key) => {
                                crate::Homogenize::convert(key, try_from)
                            }
                        }
                    }
                }

                impl<TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for OwnedKey
                where
                    voting_start::OwnedKey: crate::Homogenize<TryFrom, Error, Into>,
                {
                    fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                        match self.child {
                            OwnedSubKey::voting_start(ref key) => {
                                crate::Homogenize::convert(key, try_from)
                            }
                        }
                    }
                }

                impl<'a> From<&'a OwnedParams> for Params<'a> {
                    fn from(params: &'a OwnedParams) -> Self {
                        let OwnedParams { id, .. } = params;
                        Params { id }
                    }
                }

                impl<'a> From<Params<'a>> for OwnedParams {
                    fn from(params: Params<'a>) -> Self {
                        let Params { id, .. } = params;
                        OwnedParams { id: id.clone() }
                    }
                }

                impl<'a> From<&'a OwnedPath> for Path<'a> {
                    fn from(root: &'a OwnedPath) -> Self {
                        let OwnedPath { parent, params } = root;
                        let parent = parent.into();
                        let params = params.into();
                        Path { parent, params }
                    }
                }

                impl<'a> From<Path<'a>> for OwnedPath {
                    fn from(root: Path<'a>) -> Self {
                        let Path { parent, params } = root;
                        let parent = parent.into();
                        let params = params.into();
                        OwnedPath { parent, params }
                    }
                }

                impl<'a> From<&'a OwnedKey> for Key<'a> {
                    fn from(key: &'a OwnedKey) -> Self {
                        let OwnedKey { params, child } = key;
                        let params = params.into();
                        let child = child.into();
                        Key { params, child }
                    }
                }

                impl<'a> From<Key<'a>> for OwnedKey {
                    fn from(key: Key<'a>) -> Self {
                        let Key { params, child } = key;
                        let params = params.into();
                        let child = child.into();
                        OwnedKey { params, child }
                    }
                }

                impl<'a> From<&'a OwnedPrefix> for Prefix<'a> {
                    fn from(key: &'a OwnedPrefix) -> Self {
                        let OwnedPrefix { params, child } = key;
                        let params = params.into();
                        let child = child.as_ref().map(::core::convert::Into::into);
                        Prefix { params, child }
                    }
                }

                impl<'a> From<Prefix<'a>> for OwnedPrefix {
                    fn from(key: Prefix<'a>) -> Self {
                        let Prefix { params, child } = key;
                        let params = params.into();
                        let child = child.map(::core::convert::Into::into);
                        OwnedPrefix { params, child }
                    }
                }

                impl<'a> From<Path<'a>> for super::super::super::Prefix<'a> {
                    fn from(root: Path<'a>) -> Self {
                        let Path {
                            parent: root,
                            params,
                        } = root;
                        let key = Prefix {
                            params,
                            child: None,
                        };

                        let super::Path {
                            parent: root,
                            params,
                        } = root;
                        let key = super::Prefix {
                            params,
                            child: Some(super::SubPrefix::id(key)),
                        };

                        let super::super::Path {
                            parent: root,
                            params,
                        } = root;
                        let key = super::super::Prefix {
                            params,
                            child: Some(super::super::SubPrefix::proposal(key)),
                        };

                        let super::super::super::Path {
                            parent: super::super::super::Schema,
                            params,
                        } = root;
                        let key = super::super::super::Prefix {
                            params,
                            child: Some(super::super::super::SubPrefix::governance(key)),
                        };

                        key
                    }
                }

                impl From<OwnedPath> for super::super::super::OwnedPrefix {
                    fn from(root: OwnedPath) -> Self {
                        let OwnedPath {
                            parent: root,
                            params,
                        } = root;
                        let key = OwnedPrefix {
                            params,
                            child: None,
                        };

                        let super::OwnedPath {
                            parent: root,
                            params,
                        } = root;
                        let key = super::OwnedPrefix {
                            params,
                            child: Some(super::OwnedSubPrefix::id(key)),
                        };

                        let super::super::OwnedPath {
                            parent: root,
                            params,
                        } = root;
                        let key = super::super::OwnedPrefix {
                            params,
                            child: Some(super::super::OwnedSubPrefix::proposal(key)),
                        };

                        let super::super::super::OwnedPath {
                            params,
                            parent: super::super::super::Schema,
                        } = root;
                        let key = super::super::super::OwnedPrefix {
                            params,
                            child: Some(super::super::super::OwnedSubPrefix::governance(key)),
                        };

                        key
                    }
                }

                pub mod voting_start {
                    #[derive(
                        ::core::clone::Clone,
                        ::core::marker::Copy,
                        ::core::cmp::PartialEq,
                        ::core::cmp::Eq,
                    )]
                    pub struct Path<'a> {
                        params: Params<'a>,
                        parent: super::Path<'a>,
                    }

                    #[derive(
                        ::core::clone::Clone,
                        ::core::marker::Copy,
                        ::core::cmp::PartialEq,
                        ::core::cmp::Eq,
                    )]
                    pub(super) struct Key<'a> {
                        params: Params<'a>,
                        // special case: when leaf, no child
                    }

                    // special: when leaf, no prefix types

                    #[derive(::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq)]
                    pub struct OwnedPath {
                        params: OwnedParams,
                        parent: super::OwnedPath,
                    }

                    #[derive(
                        ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args,
                    )]
                    #[group(skip)]
                    pub(super) struct OwnedKey {
                        #[clap(flatten)]
                        params: OwnedParams,
                        // special case: when leaf, no child
                    }

                    #[derive(
                        ::core::clone::Clone,
                        ::core::marker::Copy,
                        ::core::cmp::PartialEq,
                        ::core::cmp::Eq,
                    )]
                    struct Params<'a> {
                        __: ::core::marker::PhantomData<&'a ()>,
                    }

                    #[derive(
                        ::core::clone::Clone, ::core::cmp::PartialEq, ::core::cmp::Eq, ::clap::Args,
                    )]
                    #[group(skip)]
                    struct OwnedParams {}

                    impl<Error> crate::Parse<Error> for OwnedKey
                    where
                        Error: ::core::convert::From<crate::InvalidPath>,
                    {
                        fn parse(separator: &str, segments: &[&str]) -> Result<Self, Error> {
                            match segments {
                                [] => Ok(OwnedKey {
                                    params: OwnedParams {},
                                }),
                                _ => Err(crate::InvalidPath { depth: 3 }.into()),
                            }
                        }
                    }

                    impl<'a> super::Path<'a> {
                        pub fn voting_start(self) -> Path<'a> {
                            Path {
                                parent: self,
                                params: Params {
                                    __: ::core::marker::PhantomData,
                                },
                            }
                        }
                    }

                    impl super::OwnedPath {
                        pub fn voting_start(self) -> OwnedPath {
                            OwnedPath {
                                parent: self,
                                params: OwnedParams {},
                            }
                        }
                    }

                    impl<'a> crate::DisplayPath for Key<'a> {
                        fn fmt(
                            &self,
                            separator: &str,
                            f: &mut ::core::fmt::Formatter<'_>,
                        ) -> ::core::fmt::Result {
                            let Params { .. } = &self.params;
                            write!(f, "voting_start")?;
                            Ok(())
                        }
                    }

                    impl crate::DisplayPath for OwnedKey {
                        fn fmt(
                            &self,
                            separator: &str,
                            f: &mut ::core::fmt::Formatter<'_>,
                        ) -> ::core::fmt::Result {
                            let OwnedParams { .. } = &self.params;
                            write!(f, "voting_start")?;
                            Ok(())
                        }
                    }

                    impl<'a> crate::DisplayPath for Path<'a> {
                        fn fmt(
                            &self,
                            separator: &str,
                            f: &mut ::core::fmt::Formatter<'_>,
                        ) -> ::core::fmt::Result {
                            let key: super::super::super::super::Key<'a> = (*self).into();
                            key.fmt(separator, f)
                        }
                    }

                    impl crate::DisplayPath for OwnedPath {
                        fn fmt(
                            &self,
                            separator: &str,
                            f: &mut ::core::fmt::Formatter<'_>,
                        ) -> ::core::fmt::Result {
                            let path: Path<'_> = self.into();
                            let key: super::super::super::super::Key<'_> = path.into();
                            key.fmt(separator, f)
                        }
                    }

                    impl<'a> crate::Typed for Path<'a> {
                        type Value = u64;
                    }

                    impl<'a, TryFrom, Error, Into> crate::Homogenize<TryFrom, Error, Into> for Key<'a>
                    where
                        u64: ::core::convert::TryFrom<TryFrom>,
                        Into: ::core::convert::From<u64>,
                        Error: ::core::convert::From<
                            <u64 as ::core::convert::TryFrom<TryFrom>>::Error,
                        >,
                    {
                        fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                            let value: u64 = ::core::convert::TryFrom::try_from(try_from)?;
                            Ok(value.into())
                        }
                    }

                    impl<'a> From<&'a OwnedParams> for Params<'a> {
                        fn from(params: &'a OwnedParams) -> Self {
                            let OwnedParams { .. } = params;
                            Params {
                                __: ::core::marker::PhantomData,
                            }
                        }
                    }

                    impl<'a> From<Params<'a>> for OwnedParams {
                        fn from(params: Params<'a>) -> Self {
                            let Params { .. } = params;
                            OwnedParams {}
                        }
                    }

                    impl<'a> From<&'a OwnedPath> for Path<'a> {
                        fn from(root: &'a OwnedPath) -> Self {
                            let OwnedPath { parent, params } = root;
                            let parent = parent.into();
                            let params = params.into();
                            Path { parent, params }
                        }
                    }

                    impl<'a> From<Path<'a>> for OwnedPath {
                        fn from(root: Path<'a>) -> Self {
                            let Path { parent, params } = root;
                            let parent = parent.into();
                            let params = params.into();
                            OwnedPath { parent, params }
                        }
                    }

                    impl<'a> From<&'a OwnedKey> for Key<'a> {
                        fn from(key: &'a OwnedKey) -> Self {
                            let OwnedKey { params } = key; // special: when there is no child
                            let params = params.into();
                            Key { params }
                        }
                    }

                    impl<'a> From<Key<'a>> for OwnedKey {
                        fn from(key: Key<'a>) -> Self {
                            let Key { params } = key; // special: when there is no child
                            let params = params.into();
                            OwnedKey { params }
                        }
                    }

                    impl<'a> From<Path<'a>> for super::super::super::super::Key<'a> {
                        fn from(root: Path<'a>) -> Self {
                            let Path {
                                parent: root,
                                params,
                            } = root;
                            let key = Key { params }; // special: when there is no child

                            let super::Path {
                                parent: root,
                                params,
                            } = root;
                            let key = super::Key {
                                params,
                                child: super::SubKey::voting_start(key),
                            };

                            let super::super::Path {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::Key {
                                params,
                                child: super::super::SubKey::id(key),
                            };

                            let super::super::super::Path {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::super::Key {
                                params,
                                child: super::super::super::SubKey::proposal(key),
                            };

                            let super::super::super::super::Path {
                                params,
                                parent: super::super::super::super::Schema,
                            } = root;
                            let key = super::super::super::super::Key {
                                params,
                                child: super::super::super::super::SubKey::governance(key),
                            };

                            key
                        }
                    }

                    impl From<OwnedPath> for super::super::super::super::OwnedKey {
                        fn from(root: OwnedPath) -> Self {
                            let OwnedPath {
                                parent: root,
                                params,
                            } = root;
                            let key = OwnedKey { params }; // special: when there is no child

                            let super::OwnedPath {
                                parent: root,
                                params,
                            } = root;
                            let key = super::OwnedKey {
                                params,
                                child: super::OwnedSubKey::voting_start(key),
                            };

                            let super::super::OwnedPath {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::OwnedKey {
                                params,
                                child: super::super::OwnedSubKey::id(key),
                            };

                            let super::super::super::OwnedPath {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::super::OwnedKey {
                                params,
                                child: super::super::super::OwnedSubKey::proposal(key),
                            };

                            let super::super::super::super::OwnedPath {
                                params,
                                parent: super::super::super::super::Schema,
                            } = root;
                            let key = super::super::super::super::OwnedKey {
                                params,
                                child: super::super::super::super::OwnedSubKey::governance(key),
                            };

                            key
                        }
                    }
                }
            }
        }
    }
}
