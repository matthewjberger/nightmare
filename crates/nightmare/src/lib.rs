mod app;
mod genvec;
mod graphics;
mod world;

pub mod prelude {
    pub use crate::{app::*, Duration, Instant};
    pub use egui;
    pub use log;
    pub use winit;

    #[cfg(target_arch = "wasm32")]
    pub use console_error_panic_hook::set_once as set_panic_hook;
}

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
pub use web_time::{Duration, Instant};
