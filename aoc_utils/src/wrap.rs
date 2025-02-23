pub struct WrappingWindowMut<'a, T> {
    left: &'a mut [T],
    right: &'a mut [T],
}

impl<T> WrappingWindowMut<'_, T> {
    pub fn reverse(&mut self) {
        let (left, mid, right) = if self.left.len() < self.right.len() {
            let (left, mid) = self.right.split_at_mut(self.left.len());
            (left, mid, &mut self.left[..])
        } else {
            let (mid, right) = self.left.split_at_mut(self.left.len() - self.right.len());
            (&mut self.right[..], mid, right)
        };

        debug_assert_eq!(left.len(), right.len());
        left.iter_mut()
            .zip(right.iter_mut().rev())
            .for_each(|(l, r)| std::mem::swap(l, r));
        mid.reverse();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.right.iter().chain(self.left.iter())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for WrappingWindowMut<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub trait WrappingIndex<T> {
    type Output;
    fn wrapping_window(self, begin: usize, length: usize) -> Self::Output;
}

impl<'a, T> WrappingIndex<T> for &'a mut [T] {
    type Output = WrappingWindowMut<'a, T>;

    fn wrapping_window(self, begin: usize, length: usize) -> Self::Output {
        let (head, tail) = self.split_at_mut(begin);
        let right_len = length.min(tail.len());
        let right = &mut tail[0..right_len];
        let left = &mut head[0..(length - right.len())];
        WrappingWindowMut { left, right }
    }
}

#[cfg(test)]
mod tests {
    use itertools::{Itertools, assert_equal};
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn iterate_wrapping_window() {
        let mut nums = [0, 1, 2, 3, 4, 5, 6];
        assert_equal(nums.as_mut_slice().wrapping_window(0, 3).iter(), &[0, 1, 2]);
        assert_equal(nums.as_mut_slice().wrapping_window(2, 3).iter(), &[2, 3, 4]);
        assert_equal(
            nums.as_mut_slice().wrapping_window(3, 4).iter(),
            &[3, 4, 5, 6],
        );
        assert_equal(
            nums.as_mut_slice().wrapping_window(4, 4).iter(),
            &[4, 5, 6, 0],
        );
        assert_equal(
            nums.as_mut_slice().wrapping_window(5, 4).iter(),
            &[5, 6, 0, 1],
        );
        assert_equal(
            nums.as_mut_slice().wrapping_window(6, 4).iter(),
            &[6, 0, 1, 2],
        );
    }

    #[test]
    fn reverse_wrapping_window() {
        let mut nums = [0, 1, 2, 3, 4, 5, 6];

        nums.as_mut_slice().wrapping_window(2, 3).reverse();
        assert_eq!(nums, [0, 1, 4, 3, 2, 5, 6]);

        nums.as_mut_slice().wrapping_window(3, 4).reverse();
        assert_eq!(nums, [0, 1, 4, 6, 5, 2, 3]);

        nums.as_mut_slice().wrapping_window(5, 4).reverse();
        assert_eq!(nums, [3, 2, 4, 6, 5, 1, 0]);

        nums.as_mut_slice().wrapping_window(6, 4).reverse();
        assert_eq!(nums, [2, 3, 0, 6, 5, 1, 4]);

        nums.as_mut_slice().wrapping_window(4, 4).reverse();
        assert_eq!(nums, [5, 3, 0, 6, 2, 4, 1]);
    }

    #[derive(Debug, Clone)]
    struct WrappingWindowArgs<T> {
        data: Vec<T>,
        begin: usize,
        length: usize,
    }

    prop_compose! {
        fn arbitrary_wrapping_window_args()
                (data in prop::collection::vec(i8::arbitrary(), 1..100))
                (begin in 0..data.len(), length in 0..data.len(), data in Just(data))
            -> WrappingWindowArgs<i8>
        {
            WrappingWindowArgs { data, begin, length }
        }
    }

    proptest! {
        #[test]
        fn window_reverse_does_not_panic(WrappingWindowArgs{mut data, begin, length} in arbitrary_wrapping_window_args()) {
            data.wrapping_window(begin, length).reverse();
        }

        #[test]
        fn window_reversed_list_preserves_elements(WrappingWindowArgs{mut data, begin, length} in arbitrary_wrapping_window_args()) {
            let sorted_data = data.iter().copied().sorted().collect_vec();
            data.wrapping_window(begin, length).reverse();
            data.sort();
            assert_eq!(sorted_data, data);
        }

        #[test]
        fn window_contains_elements_in_reverse_order_after_reversing(WrappingWindowArgs{mut data, begin, length} in arbitrary_wrapping_window_args()) {
            let mut window = data.wrapping_window(begin, length);
            let original_content = window.iter().copied().collect_vec();
            window.reverse();
            assert_equal(window.iter(), original_content.iter().rev());
        }

        #[test]
        fn reversing_window_has_same_effect_as_reversing_rotated_slice(WrappingWindowArgs{mut data, begin, length} in arbitrary_wrapping_window_args()) {
            let mut reversed_by_rotate = data.clone();
            reversed_by_rotate.rotate_left(begin);
            reversed_by_rotate[0..length].reverse();
            reversed_by_rotate.rotate_right(begin);

            data.wrapping_window(begin, length).reverse();

            assert_eq!(data, reversed_by_rotate);
        }
    }
}
