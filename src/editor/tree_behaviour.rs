use super::render_view::RenderViewCallback;
use super::settings::Settings;
use crate::SCREEN_SIZE;
use std::sync::{Arc, Mutex};
pub struct TreeBehavior {}

enum PaneType {
    Settings(Arc<Mutex<Settings>>),
    Render(()),
}

pub struct Pane {
    nr: usize,
    kind: PaneType,
}

impl TreeBehavior {
    pub fn create_tree(settings: Arc<Mutex<Settings>>) -> egui_tiles::Tree<Pane> {
        let mut next_view_nr = 0;
        let gen_pane = || {
            let pane = Pane {
                nr: next_view_nr,
                kind: PaneType::Settings(settings),
            };
            next_view_nr += 1;
            pane
        };

        let mut tiles = egui_tiles::Tiles::default();

        let mut tabs = vec![];

        let render_pane = Pane {
            nr: 0,
            kind: PaneType::Render(()),
        };
        tabs.push(tiles.insert_pane(render_pane));

        tabs.push(tiles.insert_pane(gen_pane()));

        // let root = tiles.insert_tab_tile(tabs);
        let root = tiles.insert_horizontal_tile(tabs);

        egui_tiles::Tree::new("strelka_tree", root, tiles)
    }
}

impl egui_tiles::Behavior<Pane> for TreeBehavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        format!("Pane {}", pane.nr).into()
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &egui_tiles::Tiles<Pane>,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        _tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            // self.add_child_to = Some(tile_id);
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut Pane,
    ) -> egui_tiles::UiResponse {
        match &pane.kind {
            PaneType::Settings(settings) => {
                ui.label("Settings.".to_string());
                // let color = settings.as_mut_slice();
                // if let Some(color_array) = color.get_mut(0..3) {
                //     ui.color_edit_button_rgb(color_array.try_into().unwrap());
                // }
                // Acquire a lock to modify settings
                if let Ok(mut settings) = settings.lock() {
                    ui.color_edit_button_rgb(settings.background_color.as_mut());
                    ui.label("background color");
                    ui.color_edit_button_rgb(settings.light_color.as_mut());
                    ui.label("light color");
                    ui.add(egui::Slider::new(&mut settings.g, -1.0..=1.0).text("g"));
                    ui.add(
                        egui::Slider::new(&mut settings.absorption, 0.0..=0.1).text("absorption"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.scattering, 0.0..=0.5).text("scattering"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.lightness, 1.0..=20.0).text("lightness"),
                    );
                    ui.add(
                        egui::Slider::new(&mut settings.ray_marching_step, 0.6..=10.0)
                            .text("ray marching step"),
                    );
                    ui.heading(format!("FPS: {:.1}", settings.fps_ctrl.get_current_fps()));
                } else {
                    ui.label("Failed to acquire settings lock.");
                }
                // ui.color_edit_button_rgb(color);
            }
            PaneType::Render(_rx) => {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let width = SCREEN_SIZE[0] as f32;
                    let height = SCREEN_SIZE[1] as f32;

                    let (rect, response) = ui.allocate_at_least(
                        //egui::Vec2::new(width, heigth - 20.0f32),
                        egui::Vec2::new(width, height),
                        egui::Sense::drag(),
                    );

                    // TODO: pass input to camera controller
                    response.has_focus();

                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        RenderViewCallback {},
                    ));
                });
            }
        }

        // You can make your pane draggable like so:
        if ui
            .add(egui::Button::new("Drag me!").sense(egui::Sense::drag()))
            .drag_started()
        {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}
