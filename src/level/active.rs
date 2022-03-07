use std::fmt::Debug;

use crate::Height;

type A<Focus> = crate::node::Active<Focus>;

pub(super) type Inner<Focus> = A<A<A<A<A<A<A<A<Focus>>>>>>>>;

#[derive(Derivative)]
#[derivative(Debug(bound = "Focus: Debug, <Focus as crate::Active>::Complete: Debug"))]
#[derivative(Clone(bound = "Focus: Clone, <Focus as crate::Active>::Complete: Clone"))]
#[derivative(PartialEq(bound = "Focus: PartialEq, <Focus as crate::Active>::Complete: PartialEq"))]
#[derivative(Eq(bound = "Focus: Eq, <Focus as crate::Active>::Complete: Eq"))]
pub(crate) struct Active<Focus: crate::Active>(Inner<Focus>);

impl<Focus: crate::Active> Height for Active<Focus>
where
    Inner<Focus>: crate::Active,
{
    type Height = <Inner<Focus> as Height>::Height;
}
