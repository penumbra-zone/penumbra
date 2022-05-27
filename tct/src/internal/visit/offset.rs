//! Internal wrapper type that offsets the index for a `Visit`, `Traverse`, or `Traversal` by a
//! particular base offset. This is used to calculate indexes during traversal.

use super::*;

pub(crate) struct Offset<T> {
    pub inner: T,
    pub offset: u64,
}

impl<T> GetHash for Offset<T>
where
    T: GetHash,
{
    #[inline]
    fn hash(&self) -> Hash {
        self.inner.hash()
    }

    #[inline]
    fn cached_hash(&self) -> Option<Hash> {
        self.inner.cached_hash()
    }
}

impl<T: Visit> Visit for Offset<T> {
    fn visit_indexed<V: Visitor>(&self, index: u64, visitor: &mut V) -> V::Output {
        self.inner.visit_indexed(index + self.offset, visitor)
    }
}

impl<S: Traverse> Traverse for Offset<S> {
    fn traverse<T: Traversal, V: Visitor>(
        &self,
        traversal: &mut T,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
    ) {
        self.inner.traverse(
            &mut Offset {
                offset: self.offset,
                inner: traversal,
            },
            visitor,
            output,
        )
    }
}

impl<S: Traversal> Traversal for Offset<&mut S> {
    fn traverse_complete<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
    ) {
        self.inner.traverse_complete(
            visitor,
            output,
            &Offset {
                inner: parent,
                offset: self.offset,
            },
            complete_children.into_iter().map(|child| Offset {
                offset: self.offset,
                inner: child,
            }),
        )
    }

    fn traverse<'a, V: Visitor, P: Visit>(
        &mut self,
        visitor: &mut V,
        output: &mut impl FnMut(V::Output),
        parent: &'a P,
        complete_children: impl IntoIterator<Item = impl Traverse>,
        frontier_child: Option<impl Traverse>,
    ) {
        self.inner.traverse(
            visitor,
            output,
            &Offset {
                inner: parent,
                offset: self.offset,
            },
            complete_children.into_iter().map(|child| Offset {
                offset: self.offset,
                inner: child,
            }),
            frontier_child.map(|child| Offset {
                offset: self.offset,
                inner: child,
            }),
        )
    }
}
