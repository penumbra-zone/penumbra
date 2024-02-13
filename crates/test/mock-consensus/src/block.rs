// TODO: see #3792.

use crate::TestNode;

struct _Builder<'e, C> {
    engine: &'e mut TestNode<C>,
}
