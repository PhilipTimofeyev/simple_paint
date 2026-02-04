#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::SimplePaintApp;
pub mod draw {
    pub mod canvas;
}

pub mod utils;

pub mod toolbar {
    pub mod main;
}
