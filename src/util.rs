use std::cmp::Ordering;

fn approx_cmp<T: PartialOrd>(a: &T, b: &T) -> Ordering {
    a.partial_cmp(b).unwrap_or(Ordering::Equal)
}

pub fn argmax<T: PartialOrd>(iter: impl Iterator<Item = T>) -> Option<usize> {
    iter.enumerate()
        .max_by(|(_, a), (_, b)| approx_cmp(a, b))
        .map(|(idx, _)| idx)
}
