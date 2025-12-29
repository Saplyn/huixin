use std::{
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub struct WithId<I, T> {
    pub id: I,
    pub item: T,
}
impl<I, T> WithId<I, T> {
    pub fn new(id: I, item: T) -> Self {
        Self { id, item }
    }
}

impl<I, T> Borrow<T> for WithId<I, T> {
    fn borrow(&self) -> &T {
        &self.item
    }
}
impl<I, T> BorrowMut<T> for WithId<I, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<I, T> AsRef<T> for WithId<I, T> {
    fn as_ref(&self) -> &T {
        &self.item
    }
}
impl<I, T> AsMut<T> for WithId<I, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<I, T> Deref for WithId<I, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
impl<I, T> DerefMut for WithId<I, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}
