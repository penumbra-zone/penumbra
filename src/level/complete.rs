use crate::{GetHash, Height};

type N<Child> = crate::node::Complete<Child>;

pub(super) type Inner<L> = complete_type!(N, L: @@@@@@@@);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complete<L>(Inner<L>);

impl<L> Height for Complete<L>
where
    Inner<L>: Height,
{
    const HEIGHT: usize = <Inner<L> as Height>::HEIGHT;
}
