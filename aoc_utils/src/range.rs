use std::ops::RangeInclusive;

/// Extension trait for set operations on range types
pub trait RangeSet: Sized {
    fn intersection(&self, other: &Self) -> Option<Self>;
}

impl<T> RangeSet for RangeInclusive<T>
where
    T: Copy + PartialOrd + Ord,
{
    fn intersection(&self, other: &Self) -> Option<Self> {
        let range = (*self.start().max(other.start()))..=(*self.end().min(other.end()));
        (!range.is_empty()).then_some(range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclusive_range_intersection() {
        // 1 2 3 4 5 6 7 8
        // ---------
        //     -----------
        assert_eq!(RangeSet::intersection(&(1..=5), &(3..=8)), Some(3..=5));

        // 1 2 3 4 5 6 7 8
        //     -----------
        // ---------
        assert_eq!(RangeSet::intersection(&(3..=8), &(1..=5)), Some(3..=5));

        // 1 2 3 4 5 6 7 8
        // ---------------
        //     -----
        assert_eq!(RangeSet::intersection(&(1..=8), &(3..=5)), Some(3..=5));

        // 1 2 3 4 5 6 7 8
        //     -----
        // ---------------
        assert_eq!(RangeSet::intersection(&(3..=5), &(1..=8)), Some(3..=5));

        // 1 2 3 4 5 6 7 8
        // -----
        //         -------
        assert_eq!(RangeSet::intersection(&(1..=3), &(5..=8)), None);

        // 1 2 3 4 5 6 7 8
        //         -------
        // -----
        assert_eq!(RangeSet::intersection(&(5..=8), &(1..=3)), None);
    }
}
