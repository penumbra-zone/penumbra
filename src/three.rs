#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Len {
    _0 = 0,
    _1 = 1,
    _2 = 2,
    _3 = 3,
}

impl Len {
    pub fn inc(&mut self) -> bool {
        match self {
            Len::_0 => *self = Len::_1,
            Len::_1 => *self = Len::_2,
            Len::_2 => *self = Len::_3,
            Len::_3 => return false,
        }
        true
    }
}

impl Default for Len {
    fn default() -> Self {
        Len::_0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Three<T> {
    len: Len,
    elems: [T; 4],
}

impl<T> Three<T> {
    pub fn new() -> Self
    where
        T: Default,
    {
        Self::default()
    }

    #[inline]
    pub fn push(mut self, item: T) -> Result<Self, [T; 4]> {
        let can_fit = self.len.inc();
        self.elems[self.len as usize - 1] = item;
        if can_fit {
            Ok(self)
        } else {
            Err(self.elems)
        }
    }

    pub fn split(&self) -> Split<T> {
        match self.len {
            Len::_0 => Split::_0([], [&self.elems[0], &self.elems[1], &self.elems[2]]),
            Len::_1 => Split::_1([&self.elems[0]], [&self.elems[1], &self.elems[2]]),
            Len::_2 => Split::_2([&self.elems[0], &self.elems[1]], [&self.elems[2]]),
            Len::_3 => Split::_3([&self.elems[0], &self.elems[1], &self.elems[2]], []),
        }
    }
}

pub enum Split<'a, T> {
    _0([&'a T; 0], [&'a T; 3]),
    _1([&'a T; 1], [&'a T; 2]),
    _2([&'a T; 2], [&'a T; 1]),
    _3([&'a T; 3], [&'a T; 0]),
}
