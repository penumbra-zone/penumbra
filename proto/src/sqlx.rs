//! SQLx support for domain types.

use crate::{DomainType, Message};

/// A wrapper newtype that provides `sqlx` support for `DomainType`s.
///
/// We can't implement `Encode` and `Decode` directly because of the orphan
/// rules, but this generic newtype allows any domain type to be wrapped into
/// something that does.  The wrapper type implements `AsRef` for the inner
/// domain type, or can be unwrapped with `into_inner`.
///
/// The name is as short as possible for ergonomics (since it will be duplicated
/// in type signatures).
pub struct S<D>(pub D);

impl<D> S<D> {
    pub fn into_inner(self) -> D {
        self.0
    }
}

impl<D> AsRef<D> for S<D> {
    fn as_ref(&self) -> &D {
        &self.0
    }
}

impl<D> From<D> for S<D> {
    fn from(d: D) -> Self {
        Self(d)
    }
}

impl<'r, D, DB> sqlx::Decode<'r, DB> for S<D>
where
    D: DomainType,
    <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
    DB: sqlx::Database,
    &'r [u8]: sqlx::Decode<'r, DB>,
{
    fn decode(
        value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(S(D::Proto::decode(<&[u8]>::decode(value)?)?
            .try_into()
            .map_err(Into::into)?))
    }
}

impl<'q, D, DB> sqlx::Encode<'q, DB> for S<D>
where
    D: DomainType,
    <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
    DB: sqlx::Database,
    Vec<u8>: sqlx::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.0.encode_to_vec().encode(buf)
    }

    fn encode(
        self,
        buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        D::Proto::from(self.0).encode_to_vec().encode(buf)
    }
}

impl<D, DB> sqlx::Type<DB> for S<D>
where
    D: DomainType,
    <D as TryFrom<D::Proto>>::Error: Into<anyhow::Error> + Send + Sync + 'static,
    DB: sqlx::Database,
    Vec<u8>: sqlx::Type<DB>,
{
    fn compatible(ty: &<DB as sqlx::Database>::TypeInfo) -> bool {
        <Vec<u8> as sqlx::Type<DB>>::compatible(ty)
    }

    fn type_info() -> <DB as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<DB>>::type_info()
    }
}
