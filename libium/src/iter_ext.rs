pub trait IterExt<T>: Iterator<Item = T>
where
    Self: Sized,
{
    fn collect_vec(self) -> Vec<T> {
        self.collect::<Vec<T>>()
    }

    fn collect_hashset(self) -> std::collections::HashSet<T>
    where
        T: Eq + std::hash::Hash,
    {
        self.collect::<std::collections::HashSet<T>>()
    }

    /// Delimits elements of `self` with a comma and returns a single string
    fn display(self, sep: impl AsRef<str>) -> String
    where
        T: ToString,
    {
        self.map(|s| ToString::to_string(&s))
            .collect_vec()
            .join(sep.as_ref())
    }
}
impl<T, U: Iterator<Item = T>> IterExt<T> for U {}

pub trait IterExtPositions<T>: Iterator<Item = (usize, T)>
where
    Self: Sized,
{
    /// Returns the indices of elements for which `predicate` returns true
    fn positions(self, predicate: impl Fn(T) -> bool) -> impl Iterator<Item = usize> {
        self.filter_map(move |(i, e)| if predicate(e) { Some(i) } else { None })
    }
}
impl<T, U: Iterator<Item = (usize, T)>> IterExtPositions<T> for U {}
