use crate::Height;

type N<Child> = crate::node::Complete<Child>;

pub(super) type Inner<Leaf> = N<N<N<N<N<N<N<N<Leaf>>>>>>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complete<L>(Inner<L>);

impl<L> Height for Complete<L>
where
    Inner<L>: Height,
{
    type Height = <Inner<L> as Height>::Height;
}
