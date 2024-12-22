use std::{collections::HashMap, hash::Hash};

type CacheFunc<T, R> = Box<dyn Fn(T, &mut CacheView<T, R>) -> R>;

struct Cache<T, R>
where
    T: Clone + Hash + Eq,
{
    data: HashMap<T, R>,
    func: CacheFunc<T, R>,
}

impl<T, R> Cache<T, R>
where
    T: Clone + Hash + Eq,
{
    pub fn new(func: impl Fn(T, &mut CacheView<T, R>) -> R + 'static) -> Self {
        Self {
            data: HashMap::new(),
            func: Box::new(func),
        }
    }

    pub fn view(&mut self) -> CacheView<'_, T, R> {
        let Cache { data, func } = self;
        CacheView { data, func }
    }
}

struct CacheView<'c, T, R>
where
    T: Clone + Hash + Eq,
{
    pub data: &'c mut HashMap<T, R>,
    pub func: &'c dyn Fn(T, &mut Self) -> R,
}

impl<T, R> CacheView<'_, T, R>
where
    T: Clone + Hash + Eq,
{
    pub fn get_or_calc(&mut self, k: T) -> &R {
        let f = self.func;
        if self.data.contains_key(&k) {
            self.data.get(&k).unwrap()
        } else {
            let v = f(k.clone(), self);
            self.data.entry(k).or_insert(v)
        }
    }
}

pub fn cached<T, R>(func: impl Fn(T, &mut dyn FnMut(T) -> R) -> R + 'static) -> impl FnMut(T) -> R
where
    T: Hash + Eq + Clone,
    R: Clone,
{
    let mut cache = Cache::new(move |x, cache: &mut CacheView<T, R>| {
        func(x, &mut |y| cache.get_or_calc(y).clone())
    });
    move |x| cache.view().get_or_calc(x.clone()).clone()
}
