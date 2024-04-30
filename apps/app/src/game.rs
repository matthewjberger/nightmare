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

impl Game {
    fn ui(&mut self, ui: &egui::Context, context: &mut app::Context) {
        self.top_bar_ui(ui);
        self.scene_tree_ui(ui, context);
    }

    fn scene_tree_ui(&mut self, ui: &egui::Context, context: &mut app::Context) {
        egui::Window::new("Scene Tree")
            .resizable(true)
            .show(ui, |ui| {
                if let Some(scene) = context.asset.scenes.first() {
                    ui.group(|ui| {
                        egui::ScrollArea::vertical()
                            .id_source(ui.next_auto_id())
                            .show(ui, |ui| {
                                node_ui(
                                    &context.asset,
                                    ui,
                                    &scene.graph,
                                    0.into(),
                                    &mut self.selected_graph_node_index,
                                );
                            });
                    });
                }
            });
    }

    fn top_bar_ui(&mut self, ui: &egui::Context) {
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ui, |ui| {
                egui::menu::bar(ui, |ui| {
                    egui::global_dark_light_mode_switch(ui);
                    ui.menu_button("File", |ui| {
                        if ui.button("Import asset (gltf/glb)...").clicked() {
                            // if let Some(path) = rfd::FileDialog::new()
                            //     .add_filter("GLTF / GLB", &["gltf", "glb"])
                            //     .pick_file()
                            // {
                            //     self.pending_messages.push(Message::Command(
                            //         Command::ImportGltfFile {
                            //             path: path.display().to_string(),
                            //         },
                            //     ));
                            //     ui.close_menu();
                            // }
                        }
                    });

                    ui.separator();
                });
            });
    }
}

impl App for Game {
    fn initialize(&mut self, context: &mut app::Context) {
        let mut asset = gltf::import_gltf_slice(include_bytes!("../glb/helmet.glb"));

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

    fn update(&mut self, context: &mut app::Context, ui: &egui::Context) {
        self.receive_messages(context);
        self.ui(ui, context);
        camera::camera_system(context);
    }
}

fn node_ui(
    asset: &asset::Asset,
    ui: &mut egui::Ui,
    graph: &asset::SceneGraph,
    graph_node_index: petgraph::graph::NodeIndex,
    selected_graph_node_index: &mut Option<petgraph::graph::NodeIndex>,
) {
    let id = ui.make_persistent_id(ui.next_auto_id());
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            let node_index = graph[graph_node_index];
            let asset::NodeMetadata { name } = &asset.metadata[node_index];
            let selected = selected_graph_node_index
                .as_ref()
                .map(|index| *index == graph_node_index)
                .unwrap_or_default();
            let response = ui.selectable_label(selected, format!("🔴 {name}"));
            if response.clicked() {
                *selected_graph_node_index = Some(graph_node_index);
            }
            response.context_menu(|ui| {
                if ui.button("Add child node").clicked() {
                    //
                }
            });
        })
        .body(|ui| {
            graph
                .neighbors_directed(graph_node_index, petgraph::Direction::Outgoing)
                .for_each(|child_index| {
                    node_ui(asset, ui, graph, child_index, selected_graph_node_index);
                });
        });
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
