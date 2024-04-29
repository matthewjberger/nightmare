use nightmare::prelude::*;

fn main() {
    launch_app(App::default());
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn run_wasm() {
    set_panic_hook();
    launch_app(App::default());
}

#[derive(Default)]
pub struct App;

impl State for App {
    fn title(&self) -> &str {
        "Nightmare"
    }

    fn initialize(&mut self, _context: &mut Context) {}

    fn receive_event(&mut self, _context: &mut Context, _event: &winit::event::Event<()>) {}

    fn update(&mut self, _context: &mut Context, ui: &egui::Context) {
        egui::Window::new("Game").show(ui, |ui| {
            ui.heading("Hello, world!");
            if ui.button("Click me!").clicked() {
                log::info!("Button clicked!");
            }
        });
    }
}
