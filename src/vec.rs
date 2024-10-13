use alloc::vec::Vec;

/// Abstracts something which can push Item into self
pub trait Push {
    /// Item stocked in the collection
    type Item;
    /// Represent a way to access Item in the collection directly after push
    type ItemView<'a>
    where
        Self: 'a;

    /// push an item into a collection, no guarantee on ordering.
    fn push<'a>(&'a mut self, item: Self::Item) -> Self::ItemView<'a>;
}

/// This is very usefull to be use on combinator like fold.
/// For example, `.fold_bounds(.., Vec::new, Acc::acc)`.
pub trait Acc {
    /// Item stocked in the collection
    type Item;

    /// Accumulate item into Self. For example, for a vector that simply a push.
    fn acc(self, item: Self::Item) -> Self;
}

impl<T> Acc for T
where
    Self: Push,
{
    type Item = <T as Push>::Item;

    fn acc(mut self, item: Self::Item) -> Self {
        self.push(item);
        self
    }
}

impl<Item> Acc for Vec<Item> {
    type Item = Item;

    fn acc<'a>(mut self, item: Self::Item) -> Self {
        self.push(item);
        self
    }
}