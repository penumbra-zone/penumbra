use crate::Height;

type C<Child> = super::super::node::Complete<Child>;

/// An eight-deep complete tree with the given leaf.
pub(super) type Inner<Leaf> = C<C<C<C<C<C<C<C<Leaf>>>>>>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complete<L>(Inner<L>);

impl<L> Height for Complete<L>
where
    Inner<L>: Height,
{
    type Height = <Inner<L> as Height>::Height;
}
