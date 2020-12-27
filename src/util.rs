use std::cmp::Ordering;

#[derive(PartialEq)]
pub struct OrdWrapper<'a, T: PartialOrd>(pub &'a T);

impl<'a, T: PartialOrd> Eq for OrdWrapper<'a, T> {}

impl<'a, T: PartialOrd> PartialOrd for OrdWrapper<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<'a, T: PartialOrd> Ord for OrdWrapper<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub fn argmax<T: PartialOrd>(iter: impl Iterator<Item = T>) -> Option<usize> {
    iter.enumerate()
        .max_by(|(_, a), (_, b)| OrdWrapper(a).cmp(&OrdWrapper(b)))
        .map(|(idx, _)| idx)
}
