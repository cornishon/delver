use std::ops::{Index, IndexMut};

use bracket_lib::terminal::Point;

/// A two dimensional rectangular grid
#[derive(Debug, Clone)]
pub struct Grid<T> {
    pub(crate) storage: Box<[T]>,
    pub width: usize,
    pub height: usize,
}

impl<T: Clone> Grid<T> {
    pub fn new(elem: T, width: usize, height: usize) -> Self {
        Grid {
            storage: vec![elem; width * height].into(),
            width,
            height,
        }
    }

    #[allow(unused)]
    pub fn fill(&mut self, value: T) {
        self.storage.iter_mut().for_each(|x| *x = value.clone());
    }
}

impl<T: Default> Grid<T> {
    pub fn default(width: usize, height: usize) -> Self {
        let mut vec = Vec::with_capacity(width * height);
        vec.resize_with(width * height, T::default);

        Grid {
            storage: vec.into_boxed_slice(),
            width,
            height,
        }
    }

    pub fn reset(&mut self) {
        self.storage.iter_mut().for_each(|x| *x = T::default());
    }
}

impl<T> Grid<T> {
    #[allow(unused)]
    pub fn to_idx(&self, x: i32, y: i32) -> usize {
        usize::try_from(y).unwrap() * self.width + usize::try_from(x).unwrap()
    }

    pub fn try_to_idx(&self, x: i32, y: i32) -> Option<usize> {
        Some(usize::try_from(y).ok()? * self.width + usize::try_from(x).ok()?)
    }

    pub fn to_pos(&self, idx: usize) -> Point {
        Point {
            x: idx as i32 % self.width as i32,
            y: idx as i32 / self.width as i32,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&T> {
        let idx = self.try_to_idx(x, y)?;
        self.storage.get(idx)
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut T> {
        let idx = self.try_to_idx(x, y)?;
        self.storage.get_mut(idx)
    }

    pub fn set(&mut self, x: i32, y: i32, value: T) {
        if let Some(x) = self.get_mut(x, y) {
            *x = value;
        }
    }
}

impl<T> Index<Point> for Grid<T> {
    type Output = T;

    fn index(&self, p: Point) -> &Self::Output {
        self.get(p.x, p.y)
            .unwrap_or_else(|| panic!("index ({}, {}) out of range", p.x, p.y))
    }
}

impl<T> IndexMut<Point> for Grid<T> {
    fn index_mut(&mut self, p: Point) -> &mut Self::Output {
        self.get_mut(p.x, p.y)
            .unwrap_or_else(|| panic!("index ({}, {}) out of range", p.x, p.y))
    }
}
