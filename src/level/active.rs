use crate::{GetHash, Height};

type A<Sibling, Focus> = crate::node::Active<Sibling, Focus>;
type N<Child> = crate::node::Complete<Child>;

pub(super) type Inner<Complete, Focus> = active_type!(A, N, Complete, Focus: @@@@@@@@);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Active<Complete, Focus>(Inner<Complete, Focus>);

impl<Sibling, Focus> Height for Active<Sibling, Focus>
where
    Inner<Sibling, Focus>: Height,
{
    const HEIGHT: usize = <Inner<Sibling, Focus> as Height>::HEIGHT;
}
