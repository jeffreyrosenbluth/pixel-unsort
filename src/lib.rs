#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::too_many_arguments)]

mod app;
pub use app::PixelUnsortApp;
pub mod art;
pub mod matrix;
pub mod sortfns;
