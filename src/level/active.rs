use std::fmt::Debug;

use crate::{Finalize, Height};

type A<Focus> = crate::node::Active<Focus>;

/// An eight-deep active tree with the given focus.
pub(super) type Inner<Focus> = A<A<A<A<A<A<A<A<Focus>>>>>>>>;

#[derive(Derivative)]
#[derivative(Debug(bound = "Focus: Debug, <Focus as Finalize>::Complete: Debug"))]
#[derivative(Clone(bound = "Focus: Clone, <Focus as Finalize>::Complete: Clone"))]
#[derivative(PartialEq(bound = "Focus: PartialEq, <Focus as Finalize>::Complete: PartialEq"))]
#[derivative(Eq(bound = "Focus: Eq, <Focus as Finalize>::Complete: Eq"))]
pub(crate) struct Active<Focus: crate::Active>(Inner<Focus>);

impl<Focus: crate::Active> Height for Active<Focus>
where
    Inner<Focus>: crate::Active,
{
    type Height = <Inner<Focus> as Height>::Height;
}
