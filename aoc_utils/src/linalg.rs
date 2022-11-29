use paste::paste;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector<T, const N: usize>(pub [T; N]);

impl<T, const N: usize> Default for Vector<T, N>
where
    [T; N]: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

macro_rules! impl_vector_op {
    ($op:ident, $func:ident) => {
        impl<T, const N: usize> std::ops::$op for Vector<T, N>
        where
        T: std::ops::$op + Copy,
        {
            type Output = Vector<<T as std::ops::$op>::Output, N>;
            fn $func(self, rhs: Vector<T, N>) -> Self::Output {
                Vector(std::array::from_fn(|i| {
                    <T as std::ops::$op>::$func(self.0[i], rhs.0[i])
                }))
            }
        }

        paste! {
            impl<T, const N: usize> std::ops::[<$op Assign>] for Vector<T, N>
            where T: std::ops::[<$op Assign>] + Copy
            {
                fn [<$func _assign>](&mut self, rhs: Vector<T, N>) {
                    self.0.iter_mut().zip(rhs.0.into_iter())
                        .for_each(|(l, r)| <T as std::ops::[<$op Assign>]>::[<$func _assign>](l, r));
                }
            }
        }
    };
}

impl_vector_op!(Add, add);
impl_vector_op!(Sub, sub);

impl<T, const N: usize> std::ops::Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.0[idx]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.0[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V1: Vector<i32, 3> = Vector([1, 2, 3]);
    const V2: Vector<i32, 3> = Vector([4, 5, 6]);
    const V3: Vector<i32, 3> = Vector([5, 7, 9]);

    #[test]
    fn zero_vector_is_default() {
        assert_eq!(Vector::<i32, 3>::default(), Vector([0, 0, 0]));
        assert_eq!(Vector::<f64, 2>::default(), Vector([0., 0.]));
    }

    #[test]
    fn vector_addition() {
        assert_eq!(V1 + V2, V3);
        let mut x = V1;
        x += V2;
        assert_eq!(x, V3);
    }

    #[test]
    fn vector_subtraction() {
        assert_eq!(V3 - V2, V1);
        let mut x = V3;
        x -= V2;
        assert_eq!(x, V1);
    }

    #[test]
    fn vector_can_be_indexed() {
        assert_eq!(V1[0], 1);
        assert_eq!(V1[1], 2);
        assert_eq!(V1[2], 3);

        let mut x = V2;
        x[1] *= -1;
        assert_eq!(x, Vector([4, -5, 6]));
    }

    #[test]
    #[should_panic]
    fn indexing_out_of_bounds_panics() {
        V1[42];
    }
}
