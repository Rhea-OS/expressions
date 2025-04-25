use alloc::vec::Vec;

pub trait Push {
    type Item;
    type ItemView<'a>
    where
        Self: 'a;

    fn push(&mut self, item: Self::Item) -> Self::ItemView<'_>;
}

pub trait Acc {
    type Item;
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

    fn acc(mut self, item: Self::Item) -> Self {
        self.push(item);
        self
    }
}