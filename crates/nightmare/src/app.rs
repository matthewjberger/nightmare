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

#[cfg(target_arch = "wasm32")]
const WASM_FIXED_WIDTH: u32 = 1920;

#[cfg(target_arch = "wasm32")]
const WASM_FIXED_HEIGHT: u32 = 1080;

async fn run_app(
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
    mut state: impl App + 'static,
) {
    let window = std::sync::Arc::new(window);

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    #[cfg(not(target_arch = "wasm32"))]
    let (width, height) = (window.inner_size().width, window.inner_size().height.min(1));

    #[cfg(target_arch = "wasm32")]
    let (width, height) = (WASM_FIXED_WIDTH, WASM_FIXED_HEIGHT);

    let mut renderer = crate::render::Renderer::new(window.clone(), width, height).await;

    let mut last_render_time = crate::Instant::now();

    let mut context = Context {
        io: Io::default(),
        delta_time: crate::Duration::default(),
        asset: crate::asset::Asset::default(),
        event_queue: Vec::new(),
    };
    state.initialize(&mut context);

    event_loop
        .run(move |event, elwt| {
            context.event_queue.drain(..).for_each(|event| match event {
                ContextEvent::RequestWorldReload => {
                    renderer.load_asset(&context.asset);
                }
                ContextEvent::Exit => {
                    elwt.exit();
                }
            });

            let (width, height, screen_descriptor) = {
                #[cfg(not(target_arch = "wasm32"))]
                let (width, height, pixels_per_point) = {
                    let window_size = window.inner_size();
                    (
                        window_size.width,
                        window_size.height.min(1),
                        window.scale_factor() as f32,
                    )
                };

                #[cfg(target_arch = "wasm32")]
                let (width, height, pixels_per_point) = (WASM_FIXED_WIDTH, WASM_FIXED_HEIGHT, 1.0);

                (
                    width,
                    height,
                    crate::render::ScreenDescriptor {
                        size_in_pixels: [width, height],
                        pixels_per_point,
                    },
                )
            };

            context.io.receive_event(
                &event,
                nalgebra_glm::vec2(width as f32 / 2.0, height as f32 / 2.0),
            );
            state.receive_event(&mut context, &event);

            state.update(&mut context);

            match event {
                winit::event::Event::AboutToWait => window.request_redraw(),

                winit::event::Event::WindowEvent { ref event, .. } => {
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
                            renderer.render_frame(&context.asset);
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
