use crate::handles::{Handle, HandleCore, Handled};
use crate::handles::collections::handle_bit_set::HandleBitSet;

impl<T> HandleBitSet<T>
where
    T: Handled,
{
    /// Get an iterator over the handles in the set.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=Handle<T>> + 'a {
        Iter::new(self)
    }
}

impl<'a, T> IntoIterator for &'a HandleBitSet<T>
where
    T: Handled,
{
    type Item = Handle<T>;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// An iterator over handles in a [HandleBitSet].
pub struct Iter<'a, T>
where
    T: Handled,
{
    set: &'a HandleBitSet<T>,
    curr_index: usize,
}

impl<'a, T> Iter<'a, T>
where
    T: Handled,
{
    pub fn new(set: &'a HandleBitSet<T>) -> Self {
        Self { set, curr_index: 0 }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Handled,
{
    type Item = Handle<T>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.curr_index >= (self.set.bytes.len() * 8) {
                break None;
            }

            let handle = Handle {
                core: T::HandleCoreType::from_index(self.curr_index),
            };
            self.curr_index += 1;

            if self.set.contains(handle) {
                break Some(handle);
            }
        }
    }
}
