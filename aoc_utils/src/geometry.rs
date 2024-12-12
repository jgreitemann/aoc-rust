pub trait Point
where
    Self: Copy + std::ops::Add<Output = Self>,
{
    fn neighbors(self) -> Neighbors<Self>;
    fn nearest_neighbors(self) -> Neighbors<Self>;
    fn next_nearest_neighbors(self) -> Neighbors<Self>;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_bounds_of_map_input() {
        assert_eq!(map_bounds("ABC\nDEF\n"), [0..3, 0..2]);
    }
}
