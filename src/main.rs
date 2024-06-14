//#![windows_subsystem = "windows"] // to turn off console.

mod focus_controller;
mod knob;
mod windows_volume_controller;

use egui::Align2;
use egui_overlay::EguiOverlay;

#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;
use crate::focus_controller::FocusController;

const VERTICAL_SPACE: f32 = 20.0;

fn main() {
    egui_overlay::start(ElectronicFocus {
        screen_width: 1920,
        screen_height: 1030,
        initialized: false,
        focus_controller: FocusController::new()
    });
}

pub struct ElectronicFocus {
    pub screen_width: i32,
    pub screen_height: i32,
    pub initialized: bool,
    pub focus_controller: FocusController
}

impl EguiOverlay for ElectronicFocus {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        self.focus_controller.tick();

        // just some controls to show how you can use glfw_backend
        egui::Window::new("Electronic Focus").anchor(Align2::RIGHT_BOTTOM, [0.0,0.0]).show(egui_context, |ui| {
            let size = glfw_backend.window_size_logical;
            let changed = false;
            if changed {
                glfw_backend.set_window_size(size);
            }

            let x_changed = false;
            let y_changed = false;
            let mut send_command = false;
            let mut get_updated_position = false;
            let mut motor_speed_str = self.focus_controller.speed.to_string();

            let temp_screen_width = self.screen_width.to_string();
            let temp_screen_height = self.screen_height.to_string();

            ui.vertical(|ui| {
                let selected_item_text = if self.focus_controller.selected_port_name == "" { "Select Port" } else { &self.focus_controller.selected_port_name };
                egui::ComboBox::from_label("Serial Port").selected_text(selected_item_text).show_ui(ui, |ui| {
                    for port in &self.focus_controller.serialports {
                        ui.selectable_value(&mut self.focus_controller.selected_port_name, port.clone(), port);
                    }
                });

                ui.add_space(VERTICAL_SPACE);

                ui.horizontal(|ui| {
                    ui.label(format!("Control mode: {}", self.focus_controller.control_mode));
                });

                ui.add_space(VERTICAL_SPACE);

                ui.horizontal(|ui| {
                    ui.label("Step Position: ");
                    ui.label(format!("{:.2}", self.focus_controller.step_position));
                });

                ui.add_space(VERTICAL_SPACE);

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.add_sized(egui::vec2(80.0, 20.0), egui::Label::new("Motor Speed"));
                    });
                    ui.add_sized(egui::vec2(50.0, 20.0), egui::TextEdit::singleline(&mut motor_speed_str));
                });

                ui.add_space(VERTICAL_SPACE);

                ui.horizontal(|ui| {
                    send_command = ui.button("Send").clicked();
                    get_updated_position = ui.button("Get Position").clicked();
                });
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
                //Initialization code goes here
                println!("Initializing USB");
                self.focus_controller.init_usb();
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

            if motor_speed_str != "" && motor_speed_str.parse::<f32>().is_ok() {
                self.focus_controller.speed = motor_speed_str.parse().unwrap();
            } else {
                self.focus_controller.speed = 0.0;
            }

            if send_command {
                self.focus_controller.move_motor();
            }

            if get_updated_position {
                self.focus_controller.get_position();
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