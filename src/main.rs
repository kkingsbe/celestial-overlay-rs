#![windows_subsystem = "windows"] // to turn off console.

mod photomanager;
mod spaceweathermanager;
extern crate reqwest;

use photomanager::PhotoManager;
use spaceweathermanager::{SpaceWeatherManager, WeatherReport};

use egui_commonmark::*;
use egui::Align2;
use egui_commonmark::CommonMarkCache;
use egui_overlay::EguiOverlay;

#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;

fn main() {
    let photo_manager = PhotoManager::new(std::time::Duration::from_secs(60 * 10), std::time::Duration::from_secs(60 * 60)); //1hr
    let space_weather_manager = SpaceWeatherManager::new(std::time::Duration::from_secs(60 * 60)); //1hr
    egui_overlay::start(CelestialOverlay { screen_width: 1920, screen_height: 1030, initialized: false, photo_manager, space_weather_manager });
}

pub struct CelestialOverlay {
    pub screen_width: i32,
    pub screen_height: i32,
    pub initialized: bool,
    pub photo_manager: PhotoManager,
    pub space_weather_manager: SpaceWeatherManager,
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

        self.space_weather_manager.refresh_if_ready();
        
        // just some controls to show how you can use glfw_backend
        egui::Window::new("Celestial Overlay").anchor(Align2::RIGHT_BOTTOM, [0.0,0.0]).show(egui_context, |ui| {
            self.space_weather_manager.weather_report.as_mut().unwrap().next_if_ready(&egui_context);

            let size = glfw_backend.window_size_logical;
            let changed = false;
            if changed {
                glfw_backend.set_window_size(size);
            }

            let mut x_changed = false;
            let mut y_changed = false;

            let mut temp_screen_width = self.screen_width.to_string();
            let mut temp_screen_height = self.screen_height.to_string();

            let mut cache = CommonMarkCache::default();
            let weather_available = self.space_weather_manager.weather_report.is_some();
            if weather_available {
                let report = self.space_weather_manager.weather_report.as_mut().unwrap();
                ui.label("Message Type: ".to_string() + &report.message_type);
                ui.label("Time: ".to_string() + &report.time);
                if report.active_image_handle.is_some() {
                    ui.add(egui::Image::new(report.active_image_handle.as_ref().unwrap()).max_width(400.));
                }
                egui::ScrollArea::vertical().max_height(200.).show(ui, |ui| {
                    CommonMarkViewer::new("viewer").show(ui, &mut cache, &report.message);
                });
            } else {
                ui.label("No space weather report available.");
            }

            if self.photo_manager.photos.len() == 0 {
                ui.label("No photos available.");
            } else {
                let mut next_clicked = false;

                ui.add_space(10.);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    ui.label("Image: ".to_string() + &self.photo_manager.photos[self.photo_manager.currrent_img].name);
                    next_clicked = ui.button("Next").on_hover_text("Next Image").clicked();
                });

                if next_clicked {
                    self.photo_manager.next();
                }
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
                self.space_weather_manager.weather_report.as_mut().unwrap().next_image(egui_context);
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