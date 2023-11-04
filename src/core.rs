use crate::matrix::*;
use std::ops::Neg;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub(crate) enum SortBy {
    Row,
    Column,
    ColRow,
    RowCol,
    Nothing,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub(crate) enum SortKey {
    Lightness,
    Hue,
    Saturation,
}

// Used to store the location of each pixel in the sort image.
pub type ImgGrid = Matrix<(usize, usize)>;

// Sort by increasing or decreasing direction of the sort function.
#[derive(Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn dir(self) -> i16 {
        match self {
            SortOrder::Ascending => 1,
            SortOrder::Descending => -1,
        }
    }
}

// Change the sort order with the unary negation operator.
impl Neg for SortOrder {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}
