pub trait Point
where
    Self: Copy + std::ops::Add<Output=Self>
{
    fn neighbors(self) -> Neighbors<Self>;
    fn nearest_neighbors(self) -> Neighbors<Self>;
}

pub struct Neighbors<P>
where P: 'static + Copy + std::ops::Add<Output=P>
{
    pub(crate) center: P,
    pub(crate) rel_iter: std::slice::Iter<'static, P>,
}

impl<P> Iterator for Neighbors<P>
where P: 'static + Copy + std::ops::Add<Output=P>
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        self.rel_iter.next().map(|r| self.center + *r)
    }
}
