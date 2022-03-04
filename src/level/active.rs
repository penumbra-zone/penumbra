type A<Sibling, Focus> = crate::node::Active<Sibling, Focus>;
type N<Child> = crate::node::Complete<Child>;

pub struct Active<L>(
    // tree depth:  1  2  3  4  5  6  7 [8]
    active_type!(A: N  N  N  N  N  N  N  N  L),
);
