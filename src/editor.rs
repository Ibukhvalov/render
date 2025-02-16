mod tree_behaviour;
mod view;
mod render_view;
mod settings;
mod fps_controller;

use std::{collections::HashSet, sync::{Arc, Mutex}};

use egui::Key;
use fps_controller::FPSController;
use glam::{Mat4, Quat, Vec3};
use log::info;
use render_view::RenderView;
use tree_behaviour::TreeBehavior;
use settings::Settings;
use view::View;

use crate::CAMERA_SPEED;


pub struct Editor {
    _viewport: Option<RenderView>,
    tree: egui_tiles::Tree<tree_behaviour::Pane>,
    settings: Arc<Mutex<Settings>>,
    camera_to_world: View,
}

impl Editor {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        width: u32,
        height: u32,
    ) -> Self {
        catppuccin_egui::set_theme(&_cc.egui_ctx, catppuccin_egui::MOCHA);

        let settings = Settings::default();
    
        let settings = Arc::new(Mutex::new(settings));

        let tree = TreeBehavior::create_tree(settings.clone());
        Self {
            _viewport: RenderView::new(_cc, width, height, settings.clone()),
            tree,
            settings: settings.clone(),
            camera_to_world: View::default(),
        }
    }

    fn handle_key_down(&mut self, keys: HashSet<Key>) {
        let rotation = self.camera_to_world.rotation_y * self.camera_to_world.rotation_x;

        let view_dir = (rotation * Vec3::Z).normalize();
        let right_dir = (rotation * Vec3::X).normalize();
        let up_dir = (Vec3::Y).normalize();
        
        for key in keys {
            self.camera_to_world.translation += match key {
                Key::W => view_dir,
                Key::S => -view_dir,
                Key::A => -right_dir,
                Key::D => right_dir,
                Key::Q => -up_dir,
                Key::E => up_dir,
                _ => Vec3::ZERO,
            } * CAMERA_SPEED;
            
            self.camera_to_world.rotation_x *= match key {
                Key::ArrowUp => Quat::from_rotation_x(-0.01f32),
                Key::ArrowDown => Quat::from_rotation_x(0.01f32),
                _ => Quat::IDENTITY,
            };

            self.camera_to_world.rotation_y *= match key {
                Key::ArrowRight => Quat::from_rotation_y(0.01f32),
                Key::ArrowLeft => Quat::from_rotation_y(-0.01f32),
                _ => Quat::IDENTITY,
            };
                
            }
        
        self.send_camera_matrix();
        
    }
 

    

    fn handle_mouse(&mut self, _pointer: egui::PointerState) {
        //self.camera_to_world.rotation_x *= Quat::from_rotation_y(pointer.delta().x * 0.002);
        //self.camera_to_world.rotation_y *= Quat::from_rotation_x(pointer.delta().y * 0.002);
    }

    fn send_camera_matrix(&self) {
        let rotation = self.camera_to_world.rotation_y * self.camera_to_world.rotation_x;
        if let Ok(mut settings) = self.settings.lock() {
            settings.matrix = Mat4::from_rotation_translation(
                rotation,
                self.camera_to_world.translation)
        }
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        info!("UPDATE");
        let input = ctx.input(|i| i.clone());
        self.handle_key_down(input.keys_down);
        self.handle_mouse(input.pointer);
        if let Ok(mut settings) = self.settings.try_lock() {
            settings.fps_ctrl.update();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                egui::menu::menu_button(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Ok(mut settings) = self.settings.lock() {
                                settings.picked_path = Some(path.display().to_string());
                            }
                        }
                    }

                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });
        // egui::SidePanel::left("tree").show(ctx, |ui| {
        //     ui.collapsing("Tree", |ui| {
        //         let tree_debug = format!("{:#?}", self.tree);
        //         ui.monospace(&tree_debug);
        //     });

        //     ui.separator();
        // });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut behavior = TreeBehavior {};
            self.tree.ui(&mut behavior, ui);
        });

        // TODO: high cpu usage here we need to repaint only render viewport
        ctx.request_repaint();
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // Give the area behind the floating windows a different color, because it looks better:
        let color = egui::lerp(
            egui::Rgba::from(visuals.panel_fill)..=egui::Rgba::from(visuals.extreme_bg_color),
            0.5,
        );
        let color = egui::Color32::from(color);
        color.to_normalized_gamma_f32()
    }
}