use std::fmt::Debug;

use crate::Height;

type A<Focus> = super::super::node::Active<Focus>;

/// An eight-deep active tree with the given focus.
pub(super) type Inner<Focus> = A<A<A<A<A<A<A<A<Focus>>>>>>>>;

#[derive(Derivative)]
#[derivative(Debug(bound = "Focus: Debug, <Focus as crate::Focus>::Complete: Debug"))]
#[derivative(Clone(bound = "Focus: Clone, <Focus as crate::Focus>::Complete: Clone"))]
#[derivative(PartialEq(bound = "Focus: PartialEq, <Focus as crate::Focus>::Complete: PartialEq"))]
#[derivative(Eq(bound = "Focus: Eq, <Focus as crate::Focus>::Complete: Eq"))]
pub struct Active<Focus: crate::Focus>(Inner<Focus>);

impl<Focus: crate::Focus> Height for Active<Focus> {
    type Height = <Inner<Focus> as Height>::Height;
}
