use std::rc::Rc;

pub trait IterUtils: Iterator {
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
