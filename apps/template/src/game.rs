use nightmare::prelude::*;

#[derive(Default)]
pub struct Game;

impl App for Game {
    fn title(&self) -> &str {
        "Nightmare"
    }

    fn initialize(&mut self, _context: &mut Context) {}

    fn receive_event(&mut self, _context: &mut Context, _event: &winit::event::Event<()>) {}

    fn update(&mut self, context: &mut Context) {
        let ui = &mut context.egui_context;
        egui::Window::new("Game").show(ui, |ui| {
            ui.heading("Hello, world!");
            if ui.button("Click me!").clicked() {
                log::info!("Button clicked!");
            }
        });
    }
}
