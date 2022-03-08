use crate::Height;

type C<Child> = super::super::node::Complete<Child>;

/// An eight-deep complete tree with the given leaf.
pub(super) type Inner<Leaf> = C<C<C<C<C<C<C<C<Leaf>>>>>>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Complete<Child>(Inner<Child>);

impl<Child: Height> Height for Complete<Child> {
    type Height = <Inner<Child> as Height>::Height;
}
