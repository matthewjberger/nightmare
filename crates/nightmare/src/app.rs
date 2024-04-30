pub fn launch_app(state: impl App + 'static) {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event()
        .build()
        .expect("Failed to create event loop");
    let mut window_builder = winit::window::WindowBuilder::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        window_builder = window_builder.with_title("Standalone Winit/Wgpu Example");
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        window_builder = window_builder.with_canvas(Some(canvas));
    }

    let window = window_builder
        .build(&event_loop)
        .expect("Failed to create window!");

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run_app(event_loop, window, state));
    }

    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run_app(event_loop, window, state));
    }
}

async fn run_app(
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
    mut state: impl App + 'static,
) {
    let window = std::sync::Arc::new(window);

    let gui_context = egui::Context::default();

    gui_context.set_pixels_per_point(window.scale_factor() as f32);
    let viewport_id = gui_context.viewport_id();
    let mut gui_state = egui_winit::State::new(
        gui_context,
        viewport_id,
        &window,
        Some(window.scale_factor() as _),
        None,
    );

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    #[cfg(not(target_arch = "wasm32"))]
    let (width, height) = (window.inner_size().width, window.inner_size().height.min(1));

    #[cfg(target_arch = "wasm32")]
    let (width, height) = (1280, 720);

    let mut renderer = crate::render::Renderer::new(window.clone(), width, height).await;

    let mut last_render_time = crate::Instant::now();

    let mut context = Context {
        io: Io::default(),
        delta_time: crate::Duration::default(),
        asset: crate::asset::Asset::default(),
        egui_context: gui_state.egui_ctx().clone(),
        event_queue: Vec::new(),
    };
    state.initialize(&mut context);

    event_loop
        .run(move |event, elwt| {
            context
                .event_queue
                .drain(..)
                .into_iter()
                .for_each(|event| match event {
                    ContextEvent::RequestWorldReload => {
                        renderer.load_asset(&context.asset);
                    }
                    ContextEvent::Exit => {
                        elwt.exit();
                    }
                });

            state.receive_event(&mut context, &event);

            match event {
                winit::event::Event::AboutToWait => window.request_redraw(),

                winit::event::Event::WindowEvent { ref event, .. } => {
                    // Receive gui window event
                    if gui_state.on_window_event(&window, event).consumed {
                        return;
                    }

                    // If the gui didn't consume the event, handle it
                    match event {
                        winit::event::WindowEvent::KeyboardInput {
                            event:
                                winit::event::KeyEvent {
                                    physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                                    ..
                                },
                            ..
                        } => {
                            // Exit by pressing the escape key
                            if matches!(key_code, winit::keyboard::KeyCode::Escape) {
                                elwt.exit();
                            }
                        }

                        // Close button handler
                        winit::event::WindowEvent::CloseRequested => {
                            log::info!("The close button was pressed; stopping");
                            elwt.exit();
                        }

                        #[cfg(not(target_arch = "wasm32"))]
                        winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize {
                            width,
                            height,
                        }) => {
                            let (width, height) = ((*width).max(1), (*height).max(1));
                            log::info!("Resizing renderer surface to: ({width}, {height})");
                            renderer.resize(width, height);
                        }

                        winit::event::WindowEvent::RedrawRequested => {
                            let now = crate::Instant::now();
                            context.delta_time = now - last_render_time;
                            last_render_time = now;

                            let gui_input = gui_state.take_egui_input(&window);
                            gui_state.egui_ctx().begin_frame(gui_input);

                            state.update(&mut context);

                            let egui::FullOutput {
                                textures_delta,
                                shapes,
                                pixels_per_point,
                                ..
                            } = gui_state.egui_ctx().end_frame();

                            let paint_jobs =
                                gui_state.egui_ctx().tessellate(shapes, pixels_per_point);

                            let screen_descriptor = {
                                let window_size = window.inner_size();
                                crate::render::ScreenDescriptor {
                                    size_in_pixels: [window_size.width, window_size.height],
                                    pixels_per_point: window.scale_factor() as f32,
                                }
                            };

                            renderer.render_frame(
                                &context.asset,
                                &screen_descriptor,
                                &paint_jobs,
                                &textures_delta,
                            );
                        }

                        _ => {}
                    }
                }

                _ => {}
            }
        })
        .unwrap();
}

pub trait App {
    fn title(&self) -> &str {
        "Nightmare"
    }

    /// Called once before the main loop
    fn initialize(&mut self, _context: &mut Context) {}

    /// Called when a winit event is received
    fn receive_event(&mut self, _context: &mut Context, _event: &winit::event::Event<()>) {}

    /// Called every frame prior to rendering
    fn update(&mut self, _context: &mut Context) {}
}

pub struct Context {
    pub io: Io,
    pub delta_time: crate::Duration,
    pub asset: crate::asset::Asset,
    pub egui_context: egui::Context,
    pub event_queue: Vec<ContextEvent>,
}

pub enum ContextEvent {
    RequestWorldReload,
    Exit,
}

#[derive(Default)]
pub struct Io {
    pub keystates: std::collections::HashMap<winit::keyboard::KeyCode, winit::event::ElementState>,
    pub mouse: Mouse,
}

impl Io {
    pub fn is_key_pressed(&self, keycode: winit::keyboard::KeyCode) -> bool {
        self.keystates.contains_key(&keycode)
            && self.keystates[&keycode] == winit::event::ElementState::Pressed
    }

    pub fn receive_event<T>(
        &mut self,
        event: &winit::event::Event<T>,
        window_center: nalgebra_glm::Vec2,
    ) {
        if let winit::event::Event::WindowEvent {
            event:
                winit::event::WindowEvent::KeyboardInput {
                    event:
                        winit::event::KeyEvent {
                            physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                            state,
                            ..
                        },
                    ..
                },
            ..
        } = *event
        {
            *self.keystates.entry(key_code).or_insert(state) = state;
        }
        self.mouse.receive_event(event, window_center);
    }
}

#[derive(Default)]
pub struct Mouse {
    pub is_left_clicked: bool,
    pub is_middle_clicked: bool,
    pub is_right_clicked: bool,
    pub position: nalgebra_glm::Vec2,
    pub position_delta: nalgebra_glm::Vec2,
    pub offset_from_center: nalgebra_glm::Vec2,
    pub wheel_delta: nalgebra_glm::Vec2,
    pub moved: bool,
    pub scrolled: bool,
}

impl Mouse {
    pub fn receive_event<T>(
        &mut self,
        event: &winit::event::Event<T>,
        window_center: nalgebra_glm::Vec2,
    ) {
        match event {
            winit::event::Event::NewEvents { .. } => self.new_events(),
            winit::event::Event::WindowEvent { event, .. } => match *event {
                winit::event::WindowEvent::MouseInput { button, state, .. } => {
                    self.mouse_input(button, state)
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    self.cursor_moved(position, window_center)
                }
                winit::event::WindowEvent::MouseWheel {
                    delta: winit::event::MouseScrollDelta::LineDelta(h_lines, v_lines),
                    ..
                } => self.mouse_wheel(h_lines, v_lines),
                _ => {}
            },
            _ => {}
        }
    }

    fn new_events(&mut self) {
        if !self.scrolled {
            self.wheel_delta = nalgebra_glm::vec2(0.0, 0.0);
        }
        self.scrolled = false;

        if !self.moved {
            self.position_delta = nalgebra_glm::vec2(0.0, 0.0);
        }
        self.moved = false;
    }

    fn cursor_moved(
        &mut self,
        position: winit::dpi::PhysicalPosition<f64>,
        window_center: nalgebra_glm::Vec2,
    ) {
        let last_position = self.position;
        let current_position = nalgebra_glm::vec2(position.x as _, position.y as _);
        self.position = current_position;
        self.position_delta = current_position - last_position;
        self.offset_from_center =
            window_center - nalgebra_glm::vec2(position.x as _, position.y as _);
        self.moved = true;
    }

    fn mouse_wheel(&mut self, h_lines: f32, v_lines: f32) {
        self.wheel_delta = nalgebra_glm::vec2(h_lines, v_lines);
        self.scrolled = true;
    }

    fn mouse_input(
        &mut self,
        button: winit::event::MouseButton,
        state: winit::event::ElementState,
    ) {
        let clicked = state == winit::event::ElementState::Pressed;
        match button {
            winit::event::MouseButton::Left => self.is_left_clicked = clicked,
            winit::event::MouseButton::Middle => self.is_middle_clicked = clicked,
            winit::event::MouseButton::Right => self.is_right_clicked = clicked,
            _ => {}
        }
    }
}
