pub mod app;
pub mod asset;
pub mod camera;
pub mod gltf;
pub mod physics;

mod render;

pub mod prelude {
    pub use crate::{
        app::{self, *},
        asset, camera, gltf, physics, Duration, Instant,
    };

    pub use egui;
    pub use log;
    pub use petgraph;
    pub use rfd;
    pub use serde;
    pub use winit;

    #[cfg(target_arch = "wasm32")]
    pub use console_error_panic_hook::set_once as set_panic_hook;
}

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
pub use web_time::{Duration, Instant};
