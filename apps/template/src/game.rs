use nightmare::prelude::*;

#[derive(Default)]
pub struct Game;

impl App for Game {
    fn title(&self) -> &str {
        "Nightmare"
    }

    fn initialize(&mut self, _context: &mut Context) {}

    fn receive_event(&mut self, _context: &mut Context, _event: &winit::event::Event<()>) {}

    fn update(&mut self, _context: &mut Context) {}
}
