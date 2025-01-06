use itertools::Itertools;

pub trait Point
where
    Self: Copy + std::ops::Add<Output = Self>,
{
    fn neighbors(self) -> Neighbors<Self>;
    fn nearest_neighbors(self) -> Neighbors<Self>;
    fn next_nearest_neighbors(self) -> Neighbors<Self>;
}

#[derive(Clone)]
pub struct Neighbors<P>
where
    P: 'static + Copy + std::ops::Add<Output = P>,
{
    pub(crate) center: P,
    pub(crate) rel_iter: std::slice::Iter<'static, P>,
}

impl<P> Iterator for Neighbors<P>
where
    P: 'static + Copy + std::ops::Add<Output = P>,
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.rel_iter.next().map(|r| self.center + *r)
    }
}

pub fn map_bounds(input: &str) -> [std::ops::Range<usize>; 2] {
    let rows = input.lines().count();
    let cols = input.lines().next().map(|line| line.len()).unwrap_or(0);
    [0..cols, 0..rows]
}

pub fn try_map_bounds<T: TryFrom<usize>>(
    input: &str,
) -> Result<[std::ops::Range<T>; 2], <T as TryFrom<usize>>::Error> {
    let rows = input.lines().count();
    let cols = input.lines().next().map(|line| line.len()).unwrap_or(0);
    Ok([
        0usize.try_into()?..cols.try_into()?,
        0usize.try_into()?..rows.try_into()?,
    ])
}

pub fn parse_ascii_map(input: &str) -> Result<ndarray::Array2<u8>, ndarray::ShapeError> {
    let mut shape = map_bounds(input).map(|b| b.end);
    shape.reverse();
    ndarray::Array2::from_shape_vec(shape, input.lines().flat_map(str::bytes).collect())
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseMapError<E> {
    #[error(transparent)]
    ShapeError(ndarray::ShapeError),
    #[error(transparent)]
    ConversionError(#[from] E),
}

pub fn try_parse_map<T, E>(
    input: &str,
    f: impl FnMut(u8) -> Result<T, E>,
) -> Result<ndarray::Array2<T>, ParseMapError<E>> {
    let mut shape = map_bounds(input).map(|b| b.end);
    shape.reverse();
    ndarray::Array2::from_shape_vec(
        shape,
        input.lines().flat_map(str::bytes).map(f).try_collect()?,
    )
    .map_err(ParseMapError::ShapeError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_bounds_of_map_input() {
        assert_eq!(map_bounds("ABC\nDEF\n"), [0..3, 0..2]);
    }

    #[test]
    fn ascii_map() {
        assert_eq!(
            parse_ascii_map("ABC\nDEF\n").unwrap(),
            ndarray::array![[b'A', b'B', b'C'], [b'D', b'E', b'F']],
        )
    }

    #[test]
    fn fallible_map() {
        #[derive(Debug, thiserror::Error, PartialEq, Eq)]
        #[error("Not a digit")]
        struct NotADigit;

        fn to_digit(ascii: u8) -> Result<u32, NotADigit> {
            char::from(ascii).to_digit(10).ok_or(NotADigit)
        }

        assert_eq!(
            try_parse_map("123\n456\n", to_digit),
            Ok(ndarray::array![[1, 2, 3], [4, 5, 6]])
        );

        assert_eq!(
            try_parse_map("123\n46\n", to_digit),
            Err(ParseMapError::ShapeError(ndarray::ShapeError::from_kind(
                ndarray::ErrorKind::OutOfBounds
            )))
        );

        assert_eq!(
            try_parse_map("123\n4x6\n", to_digit),
            Err(ParseMapError::ConversionError(NotADigit))
        );
    }
}
