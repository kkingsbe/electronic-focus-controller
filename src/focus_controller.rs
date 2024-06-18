use std::io::{BufRead, BufReader};
use std::time::Duration;
use crate::knob::{FocusEventHandler, Knob, KnobControlMode};

pub struct FocusController {
    pub speed: f32,
    speed_ui_updated: bool,
    pub step_position: f32,
    pub intended_step_position: f32,
    pub serialports: Vec<String>,
    pub selected_port_name: String,
    pub port_ref: Option<Box<dyn serialport::SerialPort>>,
    pub control_mode: String,
    knob_driver: Knob
}

impl FocusController {
    pub fn new() -> FocusController {
        return FocusController {
            speed: 0.0,
            speed_ui_updated: false,
            step_position: 0.0,
            intended_step_position: 0.0,
            serialports: Vec::new(),
            selected_port_name: String::new(),
            port_ref: None,
            control_mode: "position".to_string(),
            knob_driver: Knob::new()
        }
    }

    pub fn init_usb(&mut self) {
        let ports = serialport::available_ports().expect("No ports found!");
        self.serialports = ports.iter().map(|port| port.port_name.clone()).collect();
        self.selected_port_name = self.serialports[0].clone();
        self.knob_driver.init();
    }

    fn open_port(&mut self) {
        let port_name = self.selected_port_name.clone();
        let port = serialport::new(port_name, 9600)
            .timeout(Duration::from_millis(5000))
            .open().expect("Failed to open port");

        self.port_ref = Some(port);
    }

    pub fn set_speed(&mut self, speed: i32) {
        self.speed = speed as f32;
        self.speed_ui_updated = true;
    }

    pub fn move_motor(&mut self) {
        if self.port_ref.is_none() {
            self.open_port();
        }

        //let delta = self.intended_step_position - self.step_position;
        let mut port = self.port_ref.as_ref().unwrap().try_clone().expect("Failed to clone port");
        let command = format!("move {} {}\n", 100/*self.speed*/, self.intended_step_position);

        println!("{}", command);
        port.write(command.as_bytes()).expect("Failed to write to port");
        port.flush().unwrap();

        self.step_position = self.intended_step_position;
    }

    pub fn get_position(&mut self) {
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

    pub fn tick(&mut self) {
        self.knob_driver.tick();
        self.intended_step_position = self.knob_driver.get_setpoint() as f32;

        if self.speed_ui_updated {
            self.knob_driver.set_speed(self.speed as i32);
            self.speed_ui_updated = false;
        } else {
            self.speed = self.knob_driver.get_speed() as f32;
        }

        self.control_mode = match self.knob_driver.get_control_mode() {
            KnobControlMode::Setpoint => "setpoint".to_string(),
            KnobControlMode::Speed => "speed".to_string()
        };

        if self.intended_step_position != self.step_position {
            self.move_motor();
        }
    }
}