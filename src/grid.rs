struct As2D<T> {
    data: T,
    width: usize,
    height: usize,
}

use std::ops::{Add, Mul};

impl<T: Index<usize>> Index<Vector2<usize>> for As2D<T> {
    type Output = T::Output;
    fn index(&self, Vector2 { x, y }: Vector2<usize>) -> &Self::Output {
        &self.data[self.width * y + x]
    }
}

// These are markers for how data is laid out inside the tile map. Xs being
// "adjacent" means that if have some (x, y) at index n, then (x+1, y) is as
// index n+1, unless the next index is out of bounds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)] pub struct AdjacentX;
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)] pub struct AdjacentY;

pub type GridX<T> = Grid<T, AdjacentX>;
pub type GridY<T> = Grid<T, AdjacentY>;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Grid<T, O> {
    data: Box<[T]>,
    width: usize,
    height: usize,
    _orientation: PhantomData<O>,
}

impl<T, O> Grid<T, O> {
    pub fn from_iter<I>(iter: I, width: usize, height: usize) -> Self
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        // We just ignore the rest of the elements.
        assert!(iter.len() >= width * height);
        Grid {
            data: iter.take(width * height)
                .collect::<Vec<T>>()
                .into_boxed_slice(),
            width,
            height,
            _orientation: PhantomData,
        }
    }

    pub fn copy_grid(&mut self, other: &Self) where T: Clone {
        assert!(self.dimensions() == other.dimensions());
        for i in 0..other.data.len() {
            self.data[i] = other.data[i].clone();
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn iter(&self) -> impl Iterator<Item=&T> {
        self.data.iter()
    }
}

use cgmath::Vector2;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

impl<T> Index<Vector2<usize>> for Grid<T, AdjacentX> {
    type Output = T;
    fn index(&self, Vector2 { x, y }: Vector2<usize>) -> &Self::Output {
        assert!(x < self.width && y < self.height);
        &self.data[self.width * y + x]
    }
}

impl<T> Index<Vector2<usize>> for Grid<T, AdjacentY> {
    type Output = T;
    fn index(&self, Vector2 { x, y }: Vector2<usize>) -> &Self::Output {
        assert!(x < self.width && y < self.height);
        &self.data[self.height * x + y]
    }
}

impl<T> IndexMut<Vector2<usize>> for Grid<T, AdjacentX> {
    fn index_mut(&mut self, Vector2 { x, y }: Vector2<usize>) -> &mut Self::Output {
        assert!(x < self.width && y < self.height);
        &mut self.data[self.width * y + x]
    }
}

impl<T> IndexMut<Vector2<usize>> for Grid<T, AdjacentY> {
    fn index_mut(&mut self, Vector2 { x, y }: Vector2<usize>) -> &mut Self::Output {
        assert!(x < self.width && y < self.height);
        &mut self.data[self.height * x + y]
    }
}
