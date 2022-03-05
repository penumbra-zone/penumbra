use crate::Height;

type A<Focus> = crate::node::Active<Focus>;

pub(super) type Inner<Focus> = active_type!(A, Focus: @@@@@@@@);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Active<Focus>(Inner<Focus>);

impl<Focus> Height for Active<Focus>
where
    Inner<Focus>: crate::Height,
{
    const HEIGHT: usize = <Inner<Focus> as Height>::HEIGHT;
}
