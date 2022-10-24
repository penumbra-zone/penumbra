use core::fmt;

pub trait FormatPath {
    fn fmt(&self, separator: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub trait FormatSegment<Schema> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub struct FormatKey<'key, K: 'key>(pub &'key str, pub &'key K);

impl<'key, K: FormatPath + 'key> ::core::fmt::Display for FormatKey<'key, K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.1.fmt(self.0, f)
    }
}

pub trait Typed<'a> {
    type Value;
}

pub trait Any<TryFrom, Error, Into> {
    fn convert(&self, try_from: TryFrom) -> Result<Into, Error>;
}

// An example:

pub fn getter<'key, 'de, P, K: Typed<'key>>(
    key: K,
) -> (String, fn(&'de [u8]) -> Result<K::Value, anyhow::Error>)
where
    P: prost::Message + Default + From<<K as Typed<'key>>::Value>,
    K::Value: penumbra_proto::Protobuf<P>,
    <K::Value as TryFrom<P>>::Error: Into<anyhow::Error>,
    schema::Key<'key>: From<K>,
{
    (
        format!("{}", FormatKey("/", &schema::Key::from(key))),
        <K::Value as penumbra_proto::Protobuf<P>>::decode,
    )
}

pub fn putter<'key, P, K: Typed<'key>>(key: K, value: &K::Value) -> (String, Vec<u8>)
where
    P: prost::Message + Default + From<<K as Typed<'key>>::Value>,
    K::Value: penumbra_proto::Protobuf<P>,
    <K::Value as TryFrom<P>>::Error: Into<anyhow::Error>,
    schema::Key<'key>: From<K>,
{
    (
        format!("{}", FormatKey("/", &schema::Key::from(key))),
        penumbra_proto::Protobuf::encode_to_vec(value),
    )
}

#[test]
fn example() {
    let (path, decode) = getter(schema::governance().proposal().id(&5).voting_start());
    assert_eq!(path, "governance/proposal/5/voting_start");
}

impl FormatSegment<schema::Schema> for u64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

// pub mod schema {
//     schema! {
//          governance {
//             proposal(id: &u64) {
//                 voting_start: u64;
//             }
//         }
//     }
// }

// Generates:

pub mod schema {
    #[derive(::core::clone::Clone)]
    pub struct Schema;

    pub fn governance<'a>() -> governance::Root<'a> {
        Root {
            params: Params {
                __: ::std::marker::PhantomData,
            },
        }
        .governance()
    }

    #[derive(::core::clone::Clone)]
    pub struct Root<'a> {
        params: Params<'a>,
    }

    #[derive(::core::clone::Clone)]
    pub struct Params<'a> {
        __: ::core::marker::PhantomData<&'a ()>,
    }

    #[derive(::core::clone::Clone)]
    pub struct Prefix<'a> {
        params: Params<'a>,
        child: ::core::option::Option<SubPrefix<'a>>,
    }

    #[derive(::core::clone::Clone)]
    pub struct Key<'a> {
        params: Params<'a>,
        child: SubKey<'a>,
    }

    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    #[derive(::core::clone::Clone)]
    pub enum SubPrefix<'a> {
        governance(governance::Prefix<'a>),
    }

    #[allow(non_camel_case_types)]
    #[non_exhaustive]
    #[derive(::core::clone::Clone)]
    pub enum SubKey<'a> {
        governance(governance::Key<'a>),
    }

    impl<'a> crate::FormatPath for Key<'a> {
        fn fmt(&self, separator: &str, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match &self.child {
                SubKey::governance(child) => {
                    child.fmt(separator, f)?;
                }
            }
            Ok(())
        }
    }

    impl<'a, TryFrom, Error, Into> crate::Any<TryFrom, Error, Into> for Key<'a>
    where
        governance::Key<'a>: crate::Any<TryFrom, Error, Into>,
    {
        fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
            match self.child {
                SubKey::governance(ref key) => crate::Any::convert(key, try_from),
            }
        }
    }

    impl<'a> From<Root<'a>> for Prefix<'a> {
        fn from(root: Root<'a>) -> Self {
            let Root { params } = root;
            let key = Prefix {
                params,
                child: None,
            };

            key
        }
    }

    pub mod governance {
        #[derive(::core::clone::Clone)]
        pub struct Root<'a> {
            parent: super::Root<'a>,
            params: Params<'a>,
        }

        #[derive(::core::clone::Clone)]
        pub struct Params<'a> {
            __: ::core::marker::PhantomData<&'a ()>,
        }

        #[derive(::core::clone::Clone)]
        pub struct Prefix<'a> {
            params: Params<'a>,
            child: ::core::option::Option<SubPrefix<'a>>,
        }

        #[derive(::core::clone::Clone)]
        pub struct Key<'a> {
            params: Params<'a>,
            child: SubKey<'a>,
        }

        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        #[derive(::core::clone::Clone)]
        pub enum SubPrefix<'a> {
            proposal(proposal::Prefix<'a>),
        }

        #[allow(non_camel_case_types)]
        #[non_exhaustive]
        #[derive(::core::clone::Clone)]
        pub enum SubKey<'a> {
            proposal(proposal::Key<'a>),
        }

        impl<'a> super::Root<'a> {
            pub fn governance(self) -> Root<'a> {
                Root {
                    parent: self,
                    params: Params {
                        __: ::core::marker::PhantomData,
                    },
                }
            }
        }

        impl<'a> crate::FormatPath for Key<'a> {
            fn fmt(
                &self,
                separator: &str,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
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

        impl<'a, TryFrom, Error, Into> crate::Any<TryFrom, Error, Into> for Key<'a>
        where
            proposal::Key<'a>: crate::Any<TryFrom, Error, Into>,
        {
            fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                match self.child {
                    SubKey::proposal(ref key) => crate::Any::convert(key, try_from),
                }
            }
        }

        impl<'a> From<Root<'a>> for super::Prefix<'a> {
            fn from(root: Root<'a>) -> Self {
                let Root {
                    parent: root,
                    params,
                } = root;
                let key = Prefix {
                    params,
                    child: None,
                };

                let super::Root { params } = root;
                let key = super::Prefix {
                    params,
                    child: Some(super::SubPrefix::governance(key)),
                };

                key
            }
        }

        pub mod proposal {
            #[derive(::core::clone::Clone)]
            pub struct Root<'a> {
                parent: super::Root<'a>,
                params: Params<'a>,
            }

            #[derive(::core::clone::Clone)]
            pub struct Params<'a> {
                __: ::core::marker::PhantomData<&'a ()>,
            }

            #[derive(::core::clone::Clone)]
            pub struct Prefix<'a> {
                params: Params<'a>,
                child: ::core::option::Option<SubPrefix<'a>>,
            }

            #[derive(::core::clone::Clone)]
            pub struct Key<'a> {
                params: Params<'a>,
                child: SubKey<'a>,
            }

            #[allow(non_camel_case_types)]
            #[non_exhaustive]
            #[derive(::core::clone::Clone)]
            pub enum SubPrefix<'a> {
                id(id::Prefix<'a>),
            }

            #[allow(non_camel_case_types)]
            #[non_exhaustive]
            #[derive(::core::clone::Clone)]
            pub enum SubKey<'a> {
                id(id::Key<'a>),
            }

            impl<'a> super::Root<'a> {
                pub fn proposal(self) -> Root<'a> {
                    Root {
                        parent: self,
                        params: Params {
                            __: ::core::marker::PhantomData,
                        },
                    }
                }
            }

            impl<'a> crate::FormatPath for Key<'a> {
                fn fmt(
                    &self,
                    separator: &str,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "proposal")?;
                    write!(f, "{}", separator)?;
                    match &self.child {
                        SubKey::id(child) => {
                            child.fmt(separator, f)?;
                        }
                    }
                    Ok(())
                }
            }

            impl<'a, TryFrom, Error, Into> crate::Any<TryFrom, Error, Into> for Key<'a>
            where
                id::Key<'a>: crate::Any<TryFrom, Error, Into>,
            {
                fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                    match self.child {
                        SubKey::id(ref key) => crate::Any::convert(key, try_from),
                    }
                }
            }

            impl<'a> From<Root<'a>> for super::super::Prefix<'a> {
                fn from(root: Root<'a>) -> Self {
                    let Root {
                        parent: root,
                        params,
                    } = root;
                    let key = Prefix {
                        params,
                        child: None,
                    };

                    let super::Root {
                        parent: root,
                        params,
                    } = root;
                    let key = super::Prefix {
                        params,
                        child: Some(super::SubPrefix::proposal(key)),
                    };

                    let super::super::Root { params } = root;
                    let key = super::super::Prefix {
                        params,
                        child: Some(super::super::SubPrefix::governance(key)),
                    };

                    key
                }
            }

            pub mod id {
                #[derive(::core::clone::Clone)]
                pub struct Root<'a> {
                    parent: super::Root<'a>,
                    params: Params<'a>,
                }

                #[derive(::core::clone::Clone)]
                pub struct Params<'a> {
                    id: ::std::borrow::Cow<'a, u64>,
                }

                #[derive(::core::clone::Clone)]
                pub struct Prefix<'a> {
                    params: Params<'a>,
                    child: ::core::option::Option<SubPrefix<'a>>,
                }

                #[derive(::core::clone::Clone)]
                pub struct Key<'a> {
                    params: Params<'a>,
                    child: SubKey<'a>,
                }

                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #[derive(::core::clone::Clone)]
                pub enum SubPrefix<'a> {
                    voting_start(voting_start::Prefix<'a>),
                }

                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #[derive(::core::clone::Clone)]
                pub enum SubKey<'a> {
                    voting_start(voting_start::Key<'a>),
                }

                impl<'a> super::Root<'a> {
                    pub fn id(self, id: &'a u64) -> Root<'a> {
                        Root {
                            parent: self,
                            params: Params {
                                id: ::std::borrow::Cow::Borrowed(id),
                            },
                        }
                    }
                }

                impl<'a> crate::FormatPath for Key<'a> {
                    fn fmt(
                        &self,
                        separator: &str,
                        f: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        <u64 as crate::FormatSegment<super::super::super::Schema>>::fmt(
                            &self.params.id,
                            f,
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

                impl<'a, TryFrom, Error, Into> crate::Any<TryFrom, Error, Into> for Key<'a>
                where
                    voting_start::Key<'a>: crate::Any<TryFrom, Error, Into>,
                {
                    fn convert(&self, try_from: TryFrom) -> Result<Into, Error> {
                        match self.child {
                            SubKey::voting_start(ref key) => crate::Any::convert(key, try_from),
                        }
                    }
                }

                impl<'a> From<Root<'a>> for super::super::super::Prefix<'a> {
                    fn from(root: Root<'a>) -> Self {
                        let Root {
                            parent: root,
                            params,
                        } = root;
                        let key = Prefix {
                            params,
                            child: None,
                        };

                        let super::Root {
                            parent: root,
                            params,
                        } = root;
                        let key = super::Prefix {
                            params,
                            child: Some(super::SubPrefix::id(key)),
                        };

                        let super::super::Root {
                            parent: root,
                            params,
                        } = root;
                        let key = super::super::Prefix {
                            params,
                            child: Some(super::super::SubPrefix::proposal(key)),
                        };

                        let super::super::super::Root { params } = root;
                        let key = super::super::super::Prefix {
                            params,
                            child: Some(super::super::super::SubPrefix::governance(key)),
                        };

                        key
                    }
                }

                pub mod voting_start {
                    #[derive(::core::clone::Clone)]
                    pub struct Root<'a> {
                        parent: super::Root<'a>,
                        params: Params<'a>,
                    }

                    #[derive(::core::clone::Clone)]
                    pub struct Params<'a> {
                        __: ::core::marker::PhantomData<&'a ()>,
                    }

                    #[derive(::core::clone::Clone)]
                    pub struct Prefix<'a> {
                        params: Params<'a>,
                        child: ::core::option::Option<SubPrefix>,
                    }

                    #[derive(::core::clone::Clone)]
                    pub struct Key<'a> {
                        params: Params<'a>,
                    }

                    #[allow(non_camel_case_types)]
                    #[non_exhaustive]
                    #[derive(::core::clone::Clone)]
                    pub enum SubPrefix {}

                    impl<'a> super::Root<'a> {
                        pub fn voting_start(self) -> Root<'a> {
                            Root {
                                parent: self,
                                params: Params {
                                    __: ::core::marker::PhantomData,
                                },
                            }
                        }
                    }

                    impl<'a> crate::FormatPath for Key<'a> {
                        fn fmt(
                            &self,
                            separator: &str,
                            f: &mut ::core::fmt::Formatter<'_>,
                        ) -> ::core::fmt::Result {
                            write!(f, "voting_start")?;
                            Ok(())
                        }
                    }

                    impl<'a> crate::Typed<'a> for Root<'a> {
                        type Value = u64;
                    }

                    impl<'a, TryFrom, Error, Into> crate::Any<TryFrom, Error, Into> for Key<'a>
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

                    impl<'a> From<Root<'a>> for super::super::super::super::Prefix<'a> {
                        fn from(root: Root<'a>) -> Self {
                            let Root {
                                parent: root,
                                params,
                            } = root;
                            let key = Prefix {
                                params,
                                child: None,
                            };

                            let super::Root {
                                parent: root,
                                params,
                            } = root;
                            let key = super::Prefix {
                                params,
                                child: Some(super::SubPrefix::voting_start(key)),
                            };

                            let super::super::Root {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::Prefix {
                                params,
                                child: Some(super::super::SubPrefix::id(key)),
                            };

                            let super::super::super::Root {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::super::Prefix {
                                params,
                                child: Some(super::super::super::SubPrefix::proposal(key)),
                            };

                            let super::super::super::super::Root { params } = root;
                            let key = super::super::super::super::Prefix {
                                params,
                                child: Some(super::super::super::super::SubPrefix::governance(key)),
                            };

                            key
                        }
                    }

                    impl<'a> From<Root<'a>> for super::super::super::super::Key<'a> {
                        fn from(root: Root<'a>) -> Self {
                            let Root {
                                parent: root,
                                params,
                            } = root;
                            let key = Key { params };

                            let super::Root {
                                parent: root,
                                params,
                            } = root;
                            let key = super::Key {
                                params,
                                child: super::SubKey::voting_start(key),
                            };

                            let super::super::Root {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::Key {
                                params,
                                child: super::super::SubKey::id(key),
                            };

                            let super::super::super::Root {
                                parent: root,
                                params,
                            } = root;
                            let key = super::super::super::Key {
                                params,
                                child: super::super::super::SubKey::proposal(key),
                            };

                            let super::super::super::super::Root { params } = root;
                            let key = super::super::super::super::Key {
                                params,
                                child: super::super::super::super::SubKey::governance(key),
                            };

                            key
                        }
                    }
                }
            }
        }
    }
}
