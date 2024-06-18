extern crate hidapi;

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use hidapi::DeviceInfo;

use crate::windows_volume_controller::WindowsVolumeController;

fn get_device_ids() -> Vec<u16> {
    let api = hidapi::HidApi::new().expect("Failed to create HID API");
    return api.device_list().map(|device| device.product_id()).collect();
}

fn get_device_by_id(id: u16) -> DeviceInfo {
    let api = hidapi::HidApi::new().expect("Failed to create HID API");
    return api.device_list().find(|device| device.product_id() == id).unwrap().clone();
}

fn find_target_device() {
    println!("Scanning current devices...");
    let api = hidapi::HidApi::new().expect("Failed to create HID API");
    let initial_devices: Vec<u16> = get_device_ids();

    println!("Connect target device");
    thread::sleep(Duration::from_secs(5));

    println!("Scanning for target device...");

    let new_devices: Vec<u16> = get_device_ids();
    let new_device_id = new_devices.iter().find(|id| !initial_devices.contains(id));

    match new_device_id {
        Some(id) => {
            let device = get_device_by_id(*id);
            println!("Target device found!");
            println!("Device: {:#?}", device);
        },
        None => {
            println!("Target device not found!");
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KnobControlMode {
    Setpoint,
    Speed
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KnobCommand {
    NOP,
    MoveForwards,
    MoveBackwards,
    ModeToggle(KnobControlMode),
    DecreaseSpeed
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KnobDirection {
    Forwards,
    Backwards
}

pub struct Knob {
    speed: i32,
    speed_sensitivity: f32,
    desired_position: i32,
    last_command: KnobCommand,
    last_command_time: u64,
    command_debounce_duration: u64,
    control_mode: KnobControlMode,
    device: Option<hidapi::HidDevice>,
    volume_controller: WindowsVolumeController
}

impl Knob {
    pub fn new() -> Knob {
        return Knob {
            speed: 50,
            desired_position: 0,
            speed_sensitivity: 10.0,
            last_command: KnobCommand::NOP,
            last_command_time: 0,
            command_debounce_duration: 200, //ms
            control_mode: KnobControlMode::Setpoint,
            device: None,
            volume_controller: WindowsVolumeController::new()
        }
    }

    pub fn init(&mut self) {
        const VENDOR_ID: u16 = 19530;
        const PRODUCT_ID: u16 = 16725;

        let api = hidapi::HidApi::new().expect("Failed to create HID API");

        let device = api.open(VENDOR_ID, PRODUCT_ID).expect("Failed to open device");
        device.set_blocking_mode(false).expect("Failed to set blocking mode");

        //self.volume_controller.init();

        self.device = Some(device);
    }

    fn val_to_command(&self, value: u8) -> Option<KnobCommand> {
        match value {
            0 => Some(KnobCommand::NOP),
            1 => Some(KnobCommand::MoveForwards),
            2 => Some(KnobCommand::MoveBackwards),
            16 => Some(KnobCommand::ModeToggle(KnobControlMode::Setpoint)),
            32 => Some(KnobCommand::ModeToggle(KnobControlMode::Speed)),
            _ => None
        }
    }

    pub fn handle_command(&mut self, value: u8) {
        let command = self.val_to_command(value);

        if command.is_some() && command.unwrap() != KnobCommand::NOP {
            let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
            let time_since_last_command = current_time - self.last_command_time;

            //Make sure either the command is different from the last one or the debounce time has passed
            if command.unwrap() != self.last_command || time_since_last_command > self.command_debounce_duration {
                self.last_command = command.unwrap();
                self.last_command_time = current_time;

                println!("Command: {:#?}", command.unwrap());

                match command.unwrap() {
                    KnobCommand::MoveForwards => {
                        println!("Moving forwards");
                        match self.control_mode {
                            KnobControlMode::Setpoint => {
                                self.update_setpoint(KnobDirection::Forwards);
                                println!("Desired Position: {}", self.desired_position);
                            },
                            KnobControlMode::Speed => {
                                self.update_speed(KnobDirection::Forwards);
                                println!("Speed: {}", self.speed);
                            }
                        }
                    },
                    KnobCommand::MoveBackwards => {
                        println!("Moving backwards");
                        match self.control_mode {
                            KnobControlMode::Setpoint => {
                                self.update_setpoint(KnobDirection::Backwards);
                                println!("Desired Position: {}", self.desired_position);
                            },
                            KnobControlMode::Speed => {
                                self.update_speed(KnobDirection::Backwards);
                                println!("Speed: {}", self.speed);
                            }
                        }
                    },
                    KnobCommand::ModeToggle(KnobControlMode::Setpoint) => {
                        println!("Controlling setpoint");
                        self.control_mode = KnobControlMode::Setpoint;
                    },
                    KnobCommand::ModeToggle(KnobControlMode::Speed) => {
                        println!("Controlling speed");
                        self.control_mode = KnobControlMode::Speed;
                    },
                    _ => {}
                }
            }
        }
    }

    fn update_speed(&mut self, direction: KnobDirection) {
        self.speed += match direction {
            KnobDirection::Forwards => self.speed_sensitivity as i32,
            KnobDirection::Backwards => -self.speed_sensitivity as i32
        };
    }

    fn update_setpoint(&mut self, direction: KnobDirection) {
        self.desired_position += match direction {
            KnobDirection::Forwards => self.speed as i32,
            KnobDirection::Backwards => -self.speed as i32
        };
    }

    pub fn tick(&mut self) {
        let mut buf = [0u8];

        if self.device.is_none() {
            return;
        }

        match self.device.as_ref().unwrap().read(&mut buf) {
            Ok(size) => {
                if size > 0 {
                    let val: u8 = buf[0];
                    self.handle_command(val);
                }
            },
            Err(e) => {
                println!("Error reading from device: {:?}", e);
            }
        }
    }
}

pub trait FocusEventHandler {
    fn set_speed(&mut self, speed: i32);
    fn get_speed(&self) -> i32;
    fn get_setpoint(&self) -> i32;
    fn get_control_mode(&self) -> KnobControlMode;
}

impl FocusEventHandler for Knob {
    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
    }

    fn get_speed(&self) -> i32 {
        return self.speed;
    }

    fn get_setpoint(&self) -> i32 {
        return self.desired_position;
    }

    fn get_control_mode(&self) -> KnobControlMode {
        return self.control_mode;
    }
}

/*
fn main() {
    //find_target_device();



    let mut knob = Knob::new();

    loop {
        let mut buf = [0u8];

        match device.read(&mut buf) {
            Ok(size) => {
                if size > 0 {
                    let val: u8 = buf[0];
                    knob.handle_command(val);
                }
            },
            Err(e) => {
                println!("Error reading from device: {:?}", e);
            }
        }
    }
}
*/