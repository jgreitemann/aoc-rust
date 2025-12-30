use std::{
    collections::{BTreeMap, HashMap},
    iter::{FusedIterator, Sum},
    rc::Rc,
};

pub trait IterUtils: Iterator {
    fn try_sum<T, E>(mut self) -> Result<T, E>
    where
        Self: Sized + Iterator<Item = Result<T, E>>,
        T: Sum,
    {
        self.try_fold(std::iter::empty::<T>().sum(), |acc, val| {
            Ok(AtMostTwo::two(acc, val?).sum())
        })
    }

    fn try_unzip<A, B, E, FromA, FromB>(mut self) -> Result<(FromA, FromB), E>
    where
        Self: Sized + Iterator<Item = Result<(A, B), E>>,
        FromA: Default + Extend<A>,
        FromB: Default + Extend<B>,
    {
        self.try_fold((FromA::default(), FromB::default()), |mut colls, res| {
            colls.extend(std::iter::once(res?));
            Ok(colls)
        })
    }

    fn fallback(self, fallback_elem: Self::Item) -> Fallback<Self::Item, Self>
    where
        Self: Sized,
    {
        Fallback {
            iter: Some(self),
            fallback_elem: Some(fallback_elem),
        }
    }

    fn btree_merge<K, V>(self, mut merge_fn: impl FnMut(&mut V, V)) -> BTreeMap<K, V>
    where
        Self: Sized + Iterator<Item = (K, V)>,
        K: Ord,
    {
        self.fold(BTreeMap::new(), |mut map, (k, v)| {
            use std::collections::btree_map::Entry::*;
            match map.entry(k) {
                Vacant(vacant_entry) => {
                    vacant_entry.insert(v);
                }
                Occupied(mut occupied_entry) => merge_fn(occupied_entry.get_mut(), v),
            };

            map
        })
    }

    fn hash_merge<K, V>(self, mut merge_fn: impl FnMut(&mut V, V)) -> HashMap<K, V>
    where
        Self: Sized + Iterator<Item = (K, V)>,
        K: std::hash::Hash + Eq,
    {
        self.fold(HashMap::new(), |mut map, (k, v)| {
            use std::collections::hash_map::Entry::*;
            match map.entry(k) {
                Vacant(vacant_entry) => {
                    vacant_entry.insert(v);
                }
                Occupied(mut occupied_entry) => merge_fn(occupied_entry.get_mut(), v),
            };

            map
        })
    }
}

impl<T> IterUtils for T where T: Iterator {}

pub struct RcIter<T> {
    slice: Rc<[T]>,
    idx: usize,
}

impl<T> RcIter<T> {
    pub fn new(rc: Rc<[T]>) -> Self {
        Self { slice: rc, idx: 0 }
    }
}

impl<T: Clone> Iterator for RcIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let new_idx = self.idx + 1;
        self.slice
            .get(std::mem::replace(&mut self.idx, new_idx))
            .cloned()
    }
}

pub struct Fallback<T, I>
where
    I: Iterator<Item = T>,
{
    iter: Option<I>,
    fallback_elem: Option<T>,
}

impl<T, I> Iterator for Fallback<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.as_mut().and_then(|iter| {
            if let Some(elem) = iter.next() {
                self.fallback_elem = None;
                Some(elem)
            } else {
                self.fallback_elem.take()
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Few<T, const N: usize>([Option<T>; N]);

pub type AtMostTwo<T> = Few<T, 2>;
pub type AtMostThree<T> = Few<T, 3>;

impl<T, const N: usize> Few<T, N> {
    pub fn new<const M: usize>(items: [T; M]) -> Self {
        assert!(M <= N);
        Few(crate::array::from_iter(
            items
                .into_iter()
                .map(Some)
                .chain(std::iter::repeat_with(|| None)),
        )
        .ok()
        .unwrap())
    }

    pub fn none() -> Self {
        Few(std::array::from_fn(|_| None))
    }

    pub fn one(item: T) -> Self {
        Few::new([item])
    }

    pub fn two(item1: T, item2: T) -> Self {
        Few::new([item1, item2])
    }

    pub fn three(item1: T, item2: T, item3: T) -> Self {
        Few::new([item1, item2, item3])
    }
}

impl<T, const N: usize> Default for Few<T, N> {
    fn default() -> Self {
        Few::none()
    }
}

impl<T, const N: usize> Iterator for Few<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if N > 0 {
            let item = self.0[0].take();
            self.0.rotate_left(1);
            item
        } else {
            None
        }
    }
}

impl<T, const N: usize> FusedIterator for Few<T, N> {}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    #[should_panic]
    fn few_insufficient_capacity_one() {
        Few::<i32, 0>::one(42);
    }

    #[test]
    #[should_panic]
    fn few_insufficient_capacity_two() {
        Few::<i32, 1>::two(42, 17);
    }

    #[test]
    #[should_panic]
    fn few_insufficient_capacity_new() {
        Few::<i32, 2>::new([42, 17, -1]);
    }

    #[test]
    fn few_iterator() {
        itertools::assert_equal(Few::<i32, 0>::none(), []);
        itertools::assert_equal(Few::<i32, 1>::none(), []);
        itertools::assert_equal(Few::<i32, 2>::none(), []);
        itertools::assert_equal(Few::<i32, 1>::one(42), [42]);
        itertools::assert_equal(Few::<i32, 2>::one(42), [42]);
        itertools::assert_equal(Few::<i32, 2>::two(42, 17), [42, 17]);
        itertools::assert_equal(Few::<i32, 3>::two(42, 17), [42, 17]);
        itertools::assert_equal(Few::<i32, 3>::new([42, 17]), [42, 17]);
        itertools::assert_equal(Few::<i32, 3>::new([42, 17, -1]), [42, 17, -1]);
    }

    #[test]
    fn few_iterator_fused() {
        let mut few = AtMostTwo::two(42, 17);
        assert!(few.next().is_some());
        assert!(few.next().is_some());
        assert!(few.next().is_none());
        assert!(few.next().is_none());
        assert!(few.next().is_none());
        assert!(few.next().is_none());
        assert!(few.next().is_none());
    }

    #[test]
    fn few_default_empty() {
        itertools::assert_equal(Few::<i32, 0>::default(), []);
        itertools::assert_equal(Few::<i32, 1>::default(), []);
        itertools::assert_equal(Few::<i32, 2>::default(), []);
        itertools::assert_equal(Few::<i32, 3>::default(), []);
    }

    #[test]
    fn merge_by_counts() {
        let pi_str = format!("{:.15}", core::f64::consts::PI);
        let digits_of_pi = pi_str.chars().filter_map(|c| c.to_digit(10));
        let btree_map = digits_of_pi
            .clone()
            .map(|d| (d, 1))
            .btree_merge(|c, i| *c += i);
        let hash_map = digits_of_pi
            .clone()
            .map(|d| (d, 1))
            .hash_merge(|c, i| *c += i);
        let counts = digits_of_pi.counts();
        assert_eq!(hash_map, counts);
        itertools::assert_equal(btree_map, hash_map.into_iter().sorted());
    }
}
