use std::cmp::Ordering;

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
