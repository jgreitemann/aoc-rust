use num_traits::{Num, NumCast, Pow, Signed};
use paste::paste;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scalar<T: Num>(T);

impl<T: Num> From<T> for Scalar<T> {
    fn from(num: T) -> Self {
        Scalar(num)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vector<T: Num, const N: usize>(pub [T; N]);

impl<T: Num, const N: usize> Vector<T, N> {
    pub fn new() -> Self {
        Vector(std::array::from_fn(|_| T::zero()))
    }
}

impl<T, const N: usize> Default for Vector<T, N>
where
    T: Num,
    [T; N]: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

macro_rules! impl_vector_op {
    ($op:ident, $func:ident) => {
        impl<T, const N: usize> std::ops::$op for Vector<T, N>
        where T: Num + std::ops::$op + Copy,
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
            where T: Num + std::ops::[<$op Assign>] + Copy
            {
                fn [<$func _assign>](&mut self, rhs: Vector<T, N>) {
                    self.0.iter_mut().zip(rhs.0.into_iter())
                        .for_each(|(l, r)| <T as std::ops::[<$op Assign>]>::[<$func _assign>](l, r));
                }
            }
        }
    };
}

macro_rules! impl_scalar_op {
    ($op:ident, $func:ident) => {
        impl<T, S, const N: usize> std::ops::$op<S> for Vector<T, N>
        where
            T: Num + std::ops::$op + Copy,
            S: Into<Scalar<T>>,
        {
            type Output = Vector<<T as std::ops::$op>::Output, N>;

            fn $func(self, rhs: S) -> Self::Output {
                let scalar = rhs.into();
                Vector(std::array::from_fn(|i| {
                    <T as std::ops::$op>::$func(self.0[i], scalar.0)
                }))
            }
        }

        impl<T, const N: usize> std::ops::$op<Vector<T, N>> for Scalar<T>
        where
            T: Num + std::ops::$op + Copy,
        {
            type Output = Vector<<T as std::ops::$op>::Output, N>;

            fn $func(self, rhs: Vector<T, N>) -> Self::Output {
                Vector(std::array::from_fn(|i| {
                    <T as std::ops::$op>::$func(self.0, rhs.0[i])
                }))
            }
        }

        paste! {
            impl<T, S, const N: usize> std::ops::[<$op Assign>]<S> for Vector<T, N>
            where T: Num + std::ops::[<$op Assign>] + Copy,
            S: Into<Scalar<T>>
            {
                fn [<$func _assign>](&mut self, rhs: S) {
                    let scalar = rhs.into();
                    self.0.iter_mut()
                        .for_each(|l| <T as std::ops::[<$op Assign>]>::[<$func _assign>](l, scalar.0));
                }
            }
        }
    };
}

impl_vector_op!(Add, add);
impl_vector_op!(Sub, sub);
impl_vector_op!(Mul, mul);
impl_vector_op!(Div, div);

impl_scalar_op!(Mul, mul);
impl_scalar_op!(Div, div);

impl<T, const N: usize> std::iter::Sum for Vector<T, N>
where
    T: Copy + Num,
    [T; N]: Default,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Default::default(), std::ops::Add::add)
    }
}

impl<T: Num, const N: usize> std::ops::Deref for Vector<T, N> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Num, const N: usize> std::ops::DerefMut for Vector<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Num, const N: usize> IntoIterator for Vector<T, N> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Num, const N: usize> IntoIterator for &'a Vector<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T: Num, const N: usize> IntoIterator for &'a mut Vector<T, N> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Num + Copy + Signed,
{
    pub fn norm_l1(&self) -> T {
        self.0
            .into_iter()
            .map(num_traits::sign::abs)
            .fold(T::zero(), std::ops::Add::add)
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Num + NumCast + Copy,
{
    pub fn norm_l2(&self) -> f64 {
        f64::sqrt(NumCast::from(self.norm_l2_sq()).unwrap())
    }

    pub fn norm_l2_sq(&self) -> T {
        self.0
            .into_iter()
            .map(|x| x * x)
            .fold(T::zero(), std::ops::Add::add)
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Num + NumCast + Signed + Copy,
{
    pub fn norm_l3(&self) -> f64 {
        f64::cbrt(NumCast::from(self.norm_l3_cb()).unwrap())
    }

    pub fn norm_l3_cb(&self) -> T {
        self.0
            .into_iter()
            .map(|x| x * x * x)
            .map(num_traits::sign::abs)
            .fold(T::zero(), std::ops::Add::add)
    }
}

impl<T, const N: usize> Vector<T, N>
where
    T: Num + Pow<u8, Output = T> + NumCast + Signed + Copy,
{
    pub fn norm_lp<const P: u8>(&self) -> f64 {
        f64::powf(
            NumCast::from(self.norm_lp_pow::<P>()).unwrap(),
            1. / P as f64,
        )
    }

    pub fn norm_lp_pow<const P: u8>(&self) -> T {
        self.0
            .into_iter()
            .map(|x| x.pow(P))
            .map(num_traits::sign::abs)
            .fold(T::zero(), std::ops::Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::assert_equal;

    const V1: Vector<i32, 3> = Vector([1, 2, 3]);
    const V2: Vector<i32, 3> = Vector([4, 5, 6]);
    const V3: Vector<i32, 3> = Vector([5, 7, 9]);
    const V4: Vector<i32, 3> = Vector([4, 10, 18]);
    const V5: Vector<i32, 3> = Vector([2, 4, 6]);
    const V6: Vector<i32, 3> = Vector([10, 14, 18]);
    const VU1: Vector<u32, 3> = Vector([1, 2, 3]);
    const VF1: Vector<f32, 3> = Vector([1., 2., 3.]);

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
    fn vector_multiplication() {
        assert_eq!(V1 * V2, V4);
        let mut x = V1;
        x *= V2;
        assert_eq!(x, V4);
    }

    #[test]
    fn vector_division() {
        assert_eq!(V4 / V2, V1);
        let mut x = V4;
        x /= V2;
        assert_eq!(x, V1);
    }

    #[test]
    #[should_panic]
    fn vector_division_by_zero_panics() {
        let _ = V1 / Vector([1, 0, 1]);
    }

    #[test]
    fn vector_summation() {
        assert_eq!([V1, V2, V3].into_iter().sum::<Vector<i32, 3>>(), V6);
    }

    #[test]
    fn scalar_multiplication() {
        assert_eq!(V1 * 2, V5);
        assert_eq!(Scalar(2) * V1, V5);
        let mut x = V1;
        x *= 2;
        assert_eq!(x, V5);
    }

    #[test]
    fn scalar_division() {
        assert_eq!(V5 / 2, V1);
        let mut x = V5;
        x /= 2;
        assert_eq!(x, V1);
    }

    #[test]
    #[should_panic]
    fn scalar_division_by_zero_panics() {
        let _ = V1 / 0;
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

    #[test]
    fn vector_can_be_iterated() {
        assert_equal(V1.iter(), [&1, &2, &3]);
        assert_equal(V1.into_iter(), [1, 2, 3]);
        assert_equal((&V1).into_iter(), [&1, &2, &3]);
    }

    #[test]
    fn vector_can_be_iterated_mutably() {
        let mut x = V1;
        x.iter_mut().for_each(|e| *e = 1);
        assert_eq!(x, Vector([1; 3]));
        (&mut x).into_iter().for_each(|e| *e = 2);
        assert_eq!(x, Vector([2; 3]));
    }

    #[test]
    fn vector_integral_lp_norms() {
        assert_eq!(V1.norm_l1(), 6);

        assert_eq!(V1.norm_l2_sq(), 14);
        assert_eq!(V1.norm_l2(), f64::sqrt(14.));
        assert_eq!(VU1.norm_l2_sq(), 14);
        assert_eq!(VU1.norm_l2(), f64::sqrt(14.));

        assert_eq!(V1.norm_l3_cb(), 36);
        assert_eq!(V1.norm_l3(), f64::cbrt(36.));

        assert_eq!(V1.norm_lp_pow::<4>(), 98);
        assert_eq!(V1.norm_lp::<4>(), f64::powf(98., 0.25));
    }

    #[test]
    fn vector_floating_point_lp_norms() {
        assert_eq!(VF1.norm_l1(), 6f32);

        assert_eq!(VF1.norm_l2_sq(), 14f32);
        assert_eq!(VF1.norm_l2(), f64::sqrt(14.));

        assert_eq!(VF1.norm_l3_cb(), 36f32);
        assert_eq!(VF1.norm_l3(), f64::cbrt(36.));

        assert_eq!(VF1.norm_lp_pow::<4>(), 98f32);
        assert_eq!(VF1.norm_lp::<4>(), f64::powf(98., 0.25));
    }
}