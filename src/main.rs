#![windows_subsystem = "windows"] // to turn off console.

mod photomanager;
extern crate reqwest;

use photomanager::PhotoManager;

use egui_commonmark::*;
use egui::Align2;
use egui_commonmark::CommonMarkCache;
use egui_overlay::EguiOverlay;
#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;

fn main() {
    let mgr = PhotoManager::new(std::time::Duration::from_secs(60), std::time::Duration::from_secs(60 * 60)); //1hr
    egui_overlay::start(CelestialOverlay { screen_width: 1920, screen_height: 1030, initialized: false, photo_manager: mgr });
}

pub struct CelestialOverlay {
    pub screen_width: i32,
    pub screen_height: i32,
    pub initialized: bool,
    pub photo_manager: PhotoManager
}
impl EguiOverlay for CelestialOverlay {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        self.photo_manager.next_if_ready();
        self.photo_manager.refresh_if_ready();
        
        // just some controls to show how you can use glfw_backend
        egui::Window::new("Space Weather Report").anchor(Align2::RIGHT_BOTTOM, [0.0,0.0]).show(egui_context, |ui| {
            // sometimes, you want to see the borders to understand where the overlay is.
            /*
            let mut borders = glfw_backend.window.is_decorated();
            if ui.checkbox(&mut borders, "window borders").changed() {
                glfw_backend.window.set_decorated(borders);
            }
            */

            // how to change size.
            // WARNING: don't use drag value, because window size changing while dragging ui messes things up.
            let size = glfw_backend.window_size_logical;
            let changed = false;
            if changed {
                glfw_backend.set_window_size(size);
            }
            // how to change size.
            // WARNING: don't use drag value, because window size changing while dragging ui messes things up.

            let mut x_changed = false;
            let mut y_changed = false;

            let mut temp_screen_width = self.screen_width.to_string();
            let mut temp_screen_height = self.screen_height.to_string();

            let mut cache = CommonMarkCache::default();
            CommonMarkViewer::new("viewer").show(ui, &mut cache, "## Message Type: Space Weather Notification - Interplanetary Shock\n ## Summary:\n\nSignificant interplanetary shock detected by DSCOVR at L1 at 2024-02-24T16:16Z. \n\nThe shock is likely caused by CME with ID 2024-02-21T18:36:00-CME-001 (see notification 20240222-AL-003). Some magnetospheric compression and possible geomagnetic storm expected.\n\nActivity ID: 2024-02-24T16:16:00-IPS-001.\n\n## Notes: \n\nThis interplanetary shock arrival may possibly have additional influence from the start of a coronal hole high speed stream due to a coronal hole centered at S20W20 at the time of the arrival, but whose most western edge was observed in UV imagery to have reached S15W45 at the time of the arrival. Analysis is ongoing. \n\n\n");

            let mut next_clicked = false;
            ui.horizontal(|ui| {
                next_clicked = ui.button("Next").on_hover_text("Next Image").clicked();
            });

            if next_clicked {
                self.photo_manager.next();
            }

            ui.horizontal(|ui| {
                ui.label("x: ");
                ui.text_edit_singleline(&mut temp_screen_width);
                x_changed = ui.button("set").on_hover_text("Set window position").clicked();
            });

            ui.horizontal(|ui| {
                ui.label("y: ");
                ui.text_edit_singleline(&mut temp_screen_height);
                y_changed = ui.button("set").on_hover_text("Set window position").clicked();
            });

            if temp_screen_width != self.screen_width.to_string() {
                if temp_screen_width.parse::<i32>().is_ok() {
                    self.screen_width = temp_screen_width.parse().unwrap();
                } else {
                    self.screen_width = 0;
                }
            }

            if temp_screen_height != self.screen_height.to_string() {
                if temp_screen_height.parse::<i32>().is_ok() {
                    self.screen_height = temp_screen_height.parse().unwrap();
                } else {
                    self.screen_height = 0;
                }
            }

            if !self.initialized {
                self.initialized = true;
                glfw_backend.window.set_size(self.screen_width, self.screen_height);
            }

            glfw_backend.window.set_pos(0,0);

            if x_changed {
                glfw_backend.window.set_size(self.screen_width, glfw_backend.window_size_logical[1] as i32);
            }

            if y_changed {
                glfw_backend.window.set_size(glfw_backend.window_size_logical[0] as i32, self.screen_height);
            }
        });

        // here you decide if you want to be passthrough or not.
        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            // we need input, so we need the window to be NOT passthrough
            glfw_backend.set_passthrough(false);
        } else {
            // we don't care about input, so the window can be passthrough now
            glfw_backend.set_passthrough(true)
        }
        egui_context.request_repaint();
    }
}