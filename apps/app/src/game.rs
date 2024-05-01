use nightmare::prelude::*;

#[derive(Default)]
pub struct Game {
    pending_messages: Vec<Message>,
    selected_graph_node_index: Option<petgraph::graph::NodeIndex>,
    redo_stack: Vec<Command>,
    command_history: std::collections::VecDeque<Command>,
}

impl Game {
    fn receive_messages(&mut self, context: &mut app::Context) {
        let messages = self.pending_messages.drain(..).collect::<Vec<_>>();
        for messages in messages.into_iter() {
            match messages {
                Message::Command(command) => {
                    self.command_history.push_back(command.clone());
                    // arbitrary command history capacity
                    if self.command_history.len() == 100 {
                        self.command_history.pop_front(); // Remove the oldest element
                    }
                    match command {
                        Command::Exit => {
                            context.event_queue.push(ContextEvent::Exit);
                        }
                        Command::ImportGltfFile { path } => {
                            self.selected_graph_node_index = None;
                            self.redo_stack = Vec::new();
                            self.command_history = std::collections::VecDeque::new();
                            let name = path.to_string();

                            let mut asset = gltf::import_gltf_file(path);
                            asset.name = name;

                            if asset.scenes.is_empty() {
                                asset.scenes.push(asset::Scene::default());
                            }
                            asset.add_main_camera_to_scenegraph(0);
                            context.event_queue.push(ContextEvent::RequestWorldReload);

                            let light_node = asset.add_node();
                            asset.add_light_to_node(light_node);
                            asset.add_root_node_to_scenegraph(0, light_node);
                            context.asset = asset.clone();
                        }
                    }
                }
            }
        }
    }
}

impl App for Game {
    fn initialize(&mut self, context: &mut app::Context) {
        let mut asset = gltf::import_gltf_slice(include_bytes!("../glb/VC.glb"));

        asset.add_main_camera_to_scenegraph(0);
        context.event_queue.push(ContextEvent::RequestWorldReload);

        let light_node = asset.add_node();
        asset.add_light_to_node(light_node);
        asset.add_root_node_to_scenegraph(0, light_node);
        context.asset = asset;
    }

    fn receive_event(&mut self, context: &mut app::Context, event: &winit::event::Event<()>) {
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
            if matches!(
                (key_code, state),
                (
                    winit::keyboard::KeyCode::Escape,
                    winit::event::ElementState::Pressed
                )
            ) {
                context.event_queue.push(ContextEvent::Exit);
            }

            if matches!(
                (key_code, state),
                (
                    winit::keyboard::KeyCode::KeyF,
                    winit::event::ElementState::Pressed
                )
            ) {
                // if let Some(path) = rfd::FileDialog::new()
                //     .add_filter("GLTF / GLB", &["gltf", "glb"])
                //     .pick_file()
                // {
                //     self.pending_messages
                //         .push(Message::Command(Command::ImportGltfFile {
                //             path: path.display().to_string(),
                //         }));
                // }
            }
        }
    }

    fn update(&mut self, context: &mut app::Context) {
        self.receive_messages(context);
        camera::camera_system(context);
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    Command(Command),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(crate = "serde")]
pub enum Command {
    Exit,
    ImportGltfFile { path: String },
}
