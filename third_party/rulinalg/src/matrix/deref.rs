use super::{MatrixSlice, MatrixSliceMut};
use super::{Row, RowMut, Column, ColumnMut};

use std::ops::{Deref, DerefMut};

impl<'a, T: 'a> Deref for Row<'a, T> {
    type Target = MatrixSlice<'a, T>;

    fn deref(&self) -> &MatrixSlice<'a, T> {
        &self.row
    }
}

impl<'a, T: 'a> Deref for RowMut<'a, T> {
    type Target = MatrixSliceMut<'a, T>;

    fn deref(&self) -> &MatrixSliceMut<'a, T> {
        &self.row
    }
}

impl<'a, T: 'a> DerefMut for RowMut<'a, T> {
    fn deref_mut(&mut self) -> &mut MatrixSliceMut<'a, T> {
        &mut self.row
    }
}

impl<'a, T: 'a> Deref for Column<'a, T> {
    type Target = MatrixSlice<'a, T>;

    fn deref(&self) -> &MatrixSlice<'a, T> {
        &self.col
    }
}

impl<'a, T: 'a> Deref for ColumnMut<'a, T> {
    type Target = MatrixSliceMut<'a, T>;

    fn deref(&self) -> &MatrixSliceMut<'a, T> {
        &self.col
    }
}

impl<'a, T: 'a> DerefMut for ColumnMut<'a, T> {
    fn deref_mut(&mut self) -> &mut MatrixSliceMut<'a, T> {
        &mut self.col
    }
}
