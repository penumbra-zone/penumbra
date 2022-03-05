use crate::Height;

type A<Sibling, Focus> = crate::node::Active<Sibling, Focus>;
type N<Child> = crate::node::Complete<Child>;

pub(super) type Inner<Complete, Focus> = active_type!(A, N, Complete, Focus: @@@@@@@@);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Active<Complete, Focus>(Inner<Complete, Focus>);

impl<Complete, Focus> Height for Active<Complete, Focus>
where
    Inner<Complete, Focus>: Height,
{
    const HEIGHT: usize = <Inner<Complete, Focus> as Height>::HEIGHT;
}
