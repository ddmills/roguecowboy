use std::slice::Iter;

use serde::{Deserialize, Serialize};

// A Column-major 2D grid
#[allow(dead_code)]
#[derive(Clone, Deserialize, Serialize)]
pub struct Grid<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

#[allow(dead_code)]
impl<T> Grid<T> {
    pub fn init(width: usize, height: usize, value: T) -> Self
    where
        T: Clone,
    {
        Self {
            data: vec![value; width * height],
            width,
            height,
        }
    }

    pub fn init_fill<F>(width: usize, height: usize, mut fill_fn: F) -> Self
    where
        F: FnMut(usize, usize) -> T,
    {
        let mut data = Vec::with_capacity(width * height);

        for x in 0..width {
            for y in 0..height {
                data.push(fill_fn(x, y));
            }
        }

        Self {
            data,
            width,
            height,
        }
    }

    pub fn init_from_vec(width: usize, height: usize, data: Vec<T>) -> Self {
        Self {
            data,
            width,
            height,
        }
    }

    #[inline]
    pub fn xy(&self, idx: usize) -> (usize, usize) {
        (idx / self.height, idx % self.height)
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        x * self.height + y
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(self.idx(x, y))
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(x * self.height + y)
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.data[x * self.height + y] = value;
    }

    #[inline]
    pub fn clear(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }

    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    pub fn fill<F>(&mut self, fill_fn: F)
    where
        F: Fn(usize, usize) -> T,
    {
        for x in 0..self.width {
            for y in 0..self.height {
                self.set(x, y, fill_fn(x, y));
            }
        }
    }

    pub fn is_oob(&self, x: usize, y: usize) -> bool {
        x > 0 && y > 0 && x < self.width && y < self.height
    }

    pub fn is_on_edge(&self, x: usize, y: usize) -> bool {
        x == 0 || x == self.width - 1 || y == 0 || y == self.height - 1
    }
}
