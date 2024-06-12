//#![windows_subsystem = "windows"] // to turn off console.

use std::io::{BufRead, BufReader, Read};
use std::time::Duration;
use egui::Align2;
use egui_overlay::EguiOverlay;

#[cfg(feature = "three_d")]
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
#[cfg(feature = "wgpu")]
use egui_render_wgpu::WgpuBackend as DefaultGfxBackend;

const MAX_SPEED: f32 = 150.0;
const MAX_STEP_SIZE: i32 = 1000;

fn main() {
    egui_overlay::start(ElectronicFocus { screen_width: 1920, screen_height: 1030, initialized: false, speed: 100.0, step_position: 0.0, step_size: 0, direction: true, serialports: Vec::new(), selected_port_name: "".to_string(), port_ref: None});
}

pub struct ElectronicFocus {
    pub screen_width: i32,
    pub screen_height: i32,
    pub initialized: bool,
    pub speed: f32,
    pub step_position: f32,
    pub step_size: i32,
    pub direction: bool,
    pub serialports: Vec<String>,
    pub selected_port_name: String,
    pub port_ref: Option<Box<dyn serialport::SerialPort>>,
}

impl ElectronicFocus {
    fn init_usb(&mut self) {
        let ports = serialport::available_ports().expect("No ports found!");
        self.serialports = ports.iter().map(|port| port.port_name.clone()).collect();
        self.selected_port_name = self.serialports[0].clone();
    }

    fn open_port(&mut self) {
        let port_name = self.selected_port_name.clone();
        let port = serialport::new(port_name, 9600)
            .timeout(Duration::from_millis(5000))
            .open().expect("Failed to open port");

        self.port_ref = Some(port);
    }

    fn move_motor(&mut self, steps: i32, speed: f32) {
        if self.port_ref.is_none() {
            self.open_port();
        }

        println!("Active Port: {}", self.selected_port_name);
        let mut port = self.port_ref.as_ref().unwrap().try_clone().expect("Failed to clone port");

        let command = format!("move {} {}\n", speed, if self.direction { steps } else { -steps });
        port.write(command.as_bytes()).expect("Failed to write to port");
        port.flush().unwrap();
    }

    fn get_position(&mut self) {
        if self.port_ref.is_none() {
            self.open_port();
        }

        let mut port = self.port_ref.as_ref().unwrap().try_clone().expect("Failed to clone port");
        let command = "position\n";
        port.write(command.as_bytes()).expect("Failed to write to port");
        port.flush().unwrap();

        let mut reader = BufReader::new(port);
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        self.step_position = response.trim().parse().unwrap();
    }
}

impl EguiOverlay for ElectronicFocus {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        // just some controls to show how you can use glfw_backend
        egui::Window::new("Electronic Focus").anchor(Align2::RIGHT_BOTTOM, [0.0,0.0]).show(egui_context, |ui| {
            let size = glfw_backend.window_size_logical;
            let changed = false;
            if changed {
                glfw_backend.set_window_size(size);
            }

            let mut x_changed = false;
            let mut y_changed = false;
            let mut send_command = false;
            let mut get_updated_position = false;
            let mut reverse_direction = false;

            let mut temp_screen_width = self.screen_width.to_string();
            let mut temp_screen_height = self.screen_height.to_string();

            /*
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
            */

            ui.vertical(|ui| {
                let selected_item_text = if self.selected_port_name == "" { "Select Port" } else { &self.selected_port_name };
                egui::ComboBox::from_label("Serial Port").selected_text(selected_item_text).show_ui(ui, |ui| {
                    for port in &self.serialports {
                        ui.selectable_value(&mut self.selected_port_name, port.clone(), port);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Step Position: ");
                    ui.label(format!("{:.2}", self.step_position));
                });

                ui.checkbox(&mut self.direction, "Reverse Direction");
                ui.add(egui::Slider::new(&mut self.step_size, 0..=MAX_STEP_SIZE).text("Step Size"));
                ui.add(egui::Slider::new(&mut self.speed, 0.0..=MAX_SPEED).text("Motor Speed"));
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
                self.init_usb();
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

            if send_command {
                self.move_motor(self.step_size, self.speed);
            }

            if get_updated_position {
                self.get_position();
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