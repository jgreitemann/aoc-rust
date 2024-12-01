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
}

impl<T> IterUtils for T where T: Iterator {}
