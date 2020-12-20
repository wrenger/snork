use std::cmp::Ordering;

#[derive(PartialEq)]
pub struct OrdFloat(pub f64);

impl Eq for OrdFloat {}

impl PartialOrd for OrdFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for OrdFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub fn argmax<T: Ord>(iter: impl Iterator<Item = T>) -> Option<usize> {
    iter.enumerate()
        .max_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(idx, _)| idx)
}

pub fn argmax_f(iter: impl Iterator<Item = f64>) -> Option<usize> {
    iter.enumerate()
        .max_by(|(_, a), (_, b)| OrdFloat(*a).cmp(&OrdFloat(*b)))
        .map(|(idx, _)| idx)
}
