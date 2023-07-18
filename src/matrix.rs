#![allow(dead_code)]
use num_traits::{AsPrimitive, One, Zero};
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Matrix<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T> Matrix<T> {
    pub fn new<U: AsPrimitive<usize>>(rows: U, cols: U, data: Vec<T>) -> Self {
        assert_eq!(rows.as_() * cols.as_(), data.len());
        Self {
            width: rows.as_(),
            height: cols.as_(),
            data,
        }
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Create a new matrix using a function to generate the data.
    pub fn generate<F, U>(width: U, height: U, generator: F) -> Self
    where
        F: Fn(usize, usize) -> T,
        U: AsPrimitive<usize>,
    {
        let mut data: Vec<T> = vec![];
        for y in 0..height.as_() {
            for x in 0..width.as_() {
                data.push(generator(x, y))
            }
        }
        Matrix {
            width: width.as_(),
            height: height.as_(),
            data,
        }
    }

    /// Get a reference to the element at the given row and column.
    pub fn get_ref(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width && y < self.height {
            Some(&self.data[self.get_index(x, y)])
        } else {
            None
        }
    }

    /// Insert a value at the given row and column.
    pub fn put(&mut self, x: usize, y: usize, item: T) -> bool {
        if x >= self.width || y >= self.height {
            false
        } else {
            let idx = self.get_index(x, y);
            self.data[idx] = item;
            true
        }
    }

    /// Is this a valid row and column?
    pub fn valid<U: Into<usize>>(&self, x: U, y: U) -> bool {
        x.into() < self.width && y.into() < self.height
    }
}

impl<T> Matrix<T>
where
    T: Clone,
{
    pub fn get_row(&self, i: usize) -> Vec<T> {
        self[i].to_vec()
    }

    pub fn get_column(&self, i: usize) -> Vec<T> {
        let mut result = Vec::new();
        for j in 0..self.height {
            result.push(self[j][i].clone())
        }
        result
    }
}

impl<T> Matrix<T>
where
    T: Clone + Copy,
{
    /// Create a new matrix with the given number of rows and columns filled with
    /// a given value.
    pub fn fill(width: usize, height: usize, datum: T) -> Self {
        let data = vec![datum; width * height];
        Self {
            width,
            height,
            data,
        }
    }

    /// Return the element at the given row and column.
    pub fn get(&self, x: usize, y: usize) -> Option<T> {
        if x < self.width && y < self.height {
            let idx = self.get_index(x, y);
            Some(self.data[idx])
        } else {
            None
        }
    }
}

impl<T> Matrix<T>
where
    T: Zero + Clone,
{
    /// A matrix of all zeros.
    pub fn zeros<U: AsPrimitive<usize>>(width: U, height: U) -> Self {
        let data = vec![T::zero(); width.as_() * height.as_()];
        Self {
            width: width.as_(),
            height: height.as_(),
            data,
        }
    }
}

impl<T> Matrix<T>
where
    T: One + Clone,
{
    /// A matrix of all ones.
    pub fn ones<U: AsPrimitive<usize>>(width: U, height: U) -> Self {
        let data = vec![T::one(); width.as_() * height.as_()];
        Self {
            width: width.as_(),
            height: height.as_(),
            data,
        }
    }
}

impl<T> Index<usize> for Matrix<T> {
    type Output = [T];
    fn index(&self, index: usize) -> &Self::Output {
        let start = index * self.width;
        &self.data[start..start + self.width]
    }
}

impl<T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let start = index * self.width;
        &mut self.data[start..start + self.width]
    }
}

impl<T> PartialEq for Matrix<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.data == other.data
    }
}

impl<T> Eq for Matrix<T> where T: Eq {}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    #[test]
    fn gen_test() {
        let m = Matrix::generate(2, 3, |i, j| (i, j));
        assert_eq!(m.data, vec![(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn get_test() {
        let m = Matrix::generate(2, 3, |i, j| (i, j));
        assert_eq!(m.get(1, 1), Some((1, 1)));
        assert_eq!(m.get(2, 1), None);
        assert_eq!(m.get(0, 3), None);
    }

    #[test]
    fn get_ref_test() {
        let m = Matrix::generate(2, 3, |i, j| (i, j));
        assert_eq!(m.get_ref(1, 1), Some(&(1, 1)));
        assert_eq!(m.get_ref(2, 1), None);
        assert_eq!(m.get_ref(0, 3), None);
    }

    #[test]
    fn put_test() {
        let mut m = Matrix::generate(2, 3, |i, j| (i, j));
        assert_eq!(m.put(1, 1, (5, 5)), true);
        assert_eq!(m.get(1, 1), Some((5, 5)));
    }

    #[test]
    fn fill_test() {
        let m = Matrix::fill(1, 2, true);
        assert_eq!(m.data, vec![true, true]);
    }

    #[test]
    fn index_test() {
        let m = Matrix::generate(2, 3, |i, j| (i, j));
        assert_eq!(m[1][1], (1, 1));
    }

    #[test]
    fn indexmut_test() {
        let mut m = Matrix::generate(2, 3, |i, j| (i, j));
        m[1][1] = (5, 5);
        assert_eq!(m[1][1], (5, 5));
    }

    #[test]
    fn get_column_test() {
        let m = Matrix::generate(2, 3, |i, j| (i, j));
        let v = m.get_column(1);
        assert_eq!(v, vec![(1, 0), (1, 1), (1, 2)]);
    }
}
