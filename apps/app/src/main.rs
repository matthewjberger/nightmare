use nightmare::prelude::*;

mod game;

fn main() {
    launch_app(crate::game::Game::default());
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn run_wasm() {
    set_panic_hook();
    launch_app(crate::game::Game::default());
}
