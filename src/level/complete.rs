type N<Child> = crate::node::Complete<Child>;

pub struct Complete<L>(
    // tree depth: 1  2  3  4  5  6  7 [8]
    complete_type!(N  N  N  N  N  N  N  N  L),
);
