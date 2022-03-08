//! Vectors capable of containing at most 3 elements.

/// A vector capable of storing at most 3 elements.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Three<T> {
    elems: Vec<T>,
}

impl<T> Three<T> {
    /// Create a new `Three` with no elements, but the capacity for them pre-allocated.
    ///
    /// This technically allocates space for 4 elements, so that when [`Self::push`] overfills, it
    /// does not re-allocate.
    pub fn new() -> Self {
        Self {
            // This has capacity 4 to prevent re-allocating memory when we push to a filled `Three`
            // and thereby generate a [T; 4].
            elems: Vec::with_capacity(4),
        }
    }

    /// Push a new item into this [`Three`], or return exactly four items (including the pushed
    /// item) if the [`Three`] is already full.
    ///
    /// Note that this takes ownership of `self` and returns a new [`Three`] with the pushed item if
    /// successful.
    #[inline]
    pub fn push(mut self, item: T) -> Result<Self, [T; 4]> {
        // Push an element into the internal vec
        self.elems.push(item);
        // In debug mode, check that the size is never above 4
        debug_assert!(self.elems.len() <= 4);
        // If this makes the internal vec >= 4 elements, we're overfull
        match self.elems.try_into() {
            Ok(elems) => Err(elems),
            Err(elems) => Ok(Self { elems }),
        }
    }

    /// Get an enumeration of the elements of this [`Three`] by reference.
    pub fn elems(&self) -> Elems<T> {
        match self.elems.len() {
            0 => Elems::_0([]),
            1 => Elems::_1([&self.elems[0]]),
            2 => Elems::_2([&self.elems[0], &self.elems[1]]),
            3 => Elems::_3([&self.elems[0], &self.elems[1], &self.elems[2]]),
            _ => unreachable!("impossible for `Three` to contain more than 3 elements"),
        }
    }

    /// Convert this [`Three`] into an enumeration of its elements.
    pub fn into_elems(self) -> IntoElems<T> {
        match self.elems.len() {
            0 => IntoElems::_0([]),
            1 => IntoElems::_1(if let Ok(elems) = self.elems.try_into() {
                elems
            } else {
                unreachable!("already checked the length of elements")
            }),
            2 => IntoElems::_2(if let Ok(elems) = self.elems.try_into() {
                elems
            } else {
                unreachable!("already checked the length of elements")
            }),
            3 => IntoElems::_3(if let Ok(elems) = self.elems.try_into() {
                elems
            } else {
                unreachable!("already checked the length of elements")
            }),
            _ => unreachable!("impossible for `Three` to contain more than 3 elements"),
        }
    }
}

impl<T> Default for Three<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// All the possible cases of the elements in a [`Three`], by reference.
pub enum Elems<'a, T> {
    /// Zero elements.
    _0([&'a T; 0]),
    /// One element.
    _1([&'a T; 1]),
    /// Two elements.
    _2([&'a T; 2]),
    /// Three elements.
    _3([&'a T; 3]),
}

/// All the possible cases of the elements in a [`Three`], by value.
pub enum IntoElems<T> {
    /// Zero elements.
    _0([T; 0]),
    /// One element.
    _1([T; 1]),
    /// Two elements.
    _2([T; 2]),
    /// Three elements.
    _3([T; 3]),
}
