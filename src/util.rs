use std::cmp::Ordering;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};

fn approx_cmp<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    a.partial_cmp(b).unwrap_or(Ordering::Equal)
}

/// Returns the index of the larges element in the sequence.
///
/// # Note
/// This method may not work as expected with NaNs.
pub fn argmax<T: PartialOrd>(iter: impl Iterator<Item = T>) -> Option<usize> {
    iter.enumerate()
        .max_by(|(_, a), (_, b)| approx_cmp(a, b))
        .map(|(idx, _)| idx)
}

/// Wrapper for a key-value pair that is ordable by the key.
#[derive(Debug)]
pub struct OrdPair<K: Ord, V>(pub K, pub V);

impl<K: Ord, V> Eq for OrdPair<K, V> {}

impl<K: Ord, V> PartialEq for OrdPair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K: Ord, V> PartialOrd for OrdPair<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord, V> Ord for OrdPair<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

/// A vector with a fixed maximal length that is allocated on the stack.
pub struct FixedVec<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> FixedVec<T, N> {
    pub const fn new() -> Self {
        debug_assert!(N > 0);
        Self {
            data: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            len: 0,
        }
    }
    pub fn push(&mut self, v: T) -> bool {
        if self.len < N {
            self.data[self.len].write(v);
            self.len += 1;
            true
        } else {
            false
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            Some(unsafe { self.data[self.len].assume_init_read() })
        } else {
            None
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub const fn capacity(&self) -> usize {
        N
    }
}
impl<T, const N: usize> Drop for FixedVec<T, N> {
    fn drop(&mut self) {
        for d in self.data[..self.len].iter_mut() {
            unsafe { d.assume_init_drop() }
        }
    }
}
impl<T, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> Deref for FixedVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // TODO: replace with slice_assume_init_ref
        unsafe { &*(&self.data as *const [MaybeUninit<T>] as *const [T]) }
    }
}
impl<T, const N: usize> DerefMut for FixedVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO: replace with slice_assume_init_mut
        unsafe { &mut *(&mut self.data as *mut [MaybeUninit<T>] as *mut [T]) }
    }
}

#[cfg(test)]
mod test {
    use super::FixedVec;

    #[test]
    fn fixed_vec() {
        let mut v = FixedVec::<usize, 4>::new();
        assert_eq!(v.len(), 0);

        assert!(v.push(1));
        assert!(v.push(2));
        assert!(v.push(3));
        assert!(v.push(4));
        assert!(!v.push(42));
        assert_eq!(v.len(), 4);

        assert_eq!(v.pop(), Some(4));
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
        assert_eq!(v.len(), 0);
    }
}
