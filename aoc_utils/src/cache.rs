use std::collections::HashMap;

pub struct Cache<T, R>
where
    T: std::hash::Hash + Eq,
{
    data: HashMap<T, R>,
    func: fn(&T, &mut CacheView<T, R>) -> R,
}

impl<T, R> Cache<T, R>
where
    T: std::hash::Hash + Eq,
{
    pub fn new(func: fn(&T, &mut CacheView<T, R>) -> R) -> Self {
        Self {
            data: HashMap::new(),
            func,
        }
    }

    pub fn view(&mut self) -> CacheView<'_, T, R> {
        let Cache { data, func } = self;
        CacheView { data, func: *func }
    }
}

pub struct CacheView<'c, T, R>
where
    T: std::hash::Hash + Eq,
{
    pub data: &'c mut HashMap<T, R>,
    pub func: fn(&T, &mut Self) -> R,
}

impl<T, R> CacheView<'_, T, R>
where
    T: std::hash::Hash + Eq,
{
    pub fn get_or_calc(&mut self, k: T) -> &R {
        let f = self.func;
        if self.data.contains_key(&k) {
            self.data.get(&k).unwrap()
        } else {
            let v = f(&k, self);
            self.data.entry(k).or_insert(v)
        }
    }
}
