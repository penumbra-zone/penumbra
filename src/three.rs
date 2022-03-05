#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Three<T> {
    elems: Vec<T>,
}

impl<T> Three<T> {
    pub fn new() -> Self {
        Self {
            elems: Vec::with_capacity(4),
        }
    }

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

    pub fn elems(&self) -> Elems<T> {
        match self.elems.len() {
            0 => Elems::_0([]),
            1 => Elems::_1([&self.elems[0]]),
            2 => Elems::_2([&self.elems[0], &self.elems[1]]),
            3 => Elems::_3([&self.elems[0], &self.elems[1], &self.elems[2]]),
            _ => unreachable!("impossible for `Three` to contain more than 3 elements"),
        }
    }

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

pub enum Elems<'a, T> {
    _0([&'a T; 0]),
    _1([&'a T; 1]),
    _2([&'a T; 2]),
    _3([&'a T; 3]),
}

pub enum IntoElems<T> {
    _0([T; 0]),
    _1([T; 1]),
    _2([T; 2]),
    _3([T; 3]),
}
