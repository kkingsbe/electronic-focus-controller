use std::time::{SystemTime, UNIX_EPOCH};
use gilrs::{Gilrs, Button, Event};

pub struct AxisState {
    pub x: f32,
    pub y: f32
}

impl AxisState {
    pub fn new() -> AxisState {
        return AxisState {
            x: 0.0,
            y: 0.0
        }
    }
}

pub struct TriggerState {
    pub value: f32
}

impl TriggerState {
    pub fn new() -> TriggerState {
        return TriggerState {
            value: 0.0
        }
    }
}

pub struct GamepadDriver {
    gilrs: Gilrs,
    rt_state: TriggerState,
    lt_state: TriggerState,
    left_joystick_state: AxisState,
    speed: f32,
    setpoint: f32,
    last_time: u64,
    dt: u64,
    joystick_deadzone: f32,
    speed_curvature: f32
}

pub trait FocusEventHandler {
    fn set_speed(&mut self, speed: f32);
    fn set_setpoint(&mut self, setpoint: f32);
    fn get_speed(&self) -> f32;
    fn get_setpoint(&self) -> f32;
}

impl FocusEventHandler for GamepadDriver {
    fn set_speed(&mut self, speed: f32) {
        println!("Setting speed to {}", speed);
        self.speed = speed;
    }

    fn set_setpoint(&mut self, setpoint: f32) {
        println!("Setting setpoint to {}", setpoint);
        self.setpoint = setpoint;
    }

    fn get_speed(&self) -> f32 {
        return self.speed;
    }

    fn get_setpoint(&self) -> f32 {
        return self.setpoint;
    }
}

impl GamepadDriver {
    pub fn new() -> GamepadDriver {
        return GamepadDriver {
            gilrs: Gilrs::new().unwrap(),
            rt_state: TriggerState::new(),
            lt_state: TriggerState::new(),
            left_joystick_state: AxisState::new(),
            speed: 0.0,
            setpoint: 0.0,
            last_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            dt: 0,
            joystick_deadzone: 0.1,
            speed_curvature: 20.0
        }
    }

    pub fn init(&mut self) {
        println!("Gamepad driver initialized");
    }

    pub fn tick(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        self.dt = now - self.last_time;
        self.last_time = now;

        //Get updated axis positions
        self.handle_events();

        self.set_speed(self.calculate_speed());
        self.set_setpoint(self.calculate_setpoint());
    }

    fn calculate_speed(&self) -> f32 {
        let mut remapped_position = GamepadDriver::sigmoid(self.speed_curvature, self.left_joystick_state.y);
        if remapped_position.abs() < self.joystick_deadzone {
            remapped_position = 0.0;
        }

        let delta = remapped_position * 0.1 * self.dt as f32;
        let new_speed = self.get_speed() + delta;
        if new_speed < 0.0 {
            return 0.0;
        } else {
            return new_speed;
        }
    }

    fn calculate_setpoint(&self) -> f32 {
        let mut increasing = false;
        
        if self.rt_state.value > 0.0 {
            increasing = true;
        }

        let position: f32;

        if increasing {
            position = self.rt_state.value;
        } else {
            position = -self.lt_state.value;
        }

        let multiplier = 0.01 * self.dt as f32 * self.get_speed();
        let delta = position * multiplier;

        return self.get_setpoint() + delta;
    }

    fn sigmoid(steepness: f32, input: f32) -> f32 {
        let was_negative = input < 0.0;
        let positive_input = input.abs();
        let evaluated = 1.0 / (1.0 + (-steepness * (positive_input - 0.5)).exp());
        if was_negative {
            return -evaluated;
        } else {
            return evaluated;
        }
    }

    fn exponential(steepness: f32, input: f32) -> f32 {
        return input.signum() * (1.0 - (-steepness * input.abs()).exp());
    }

    fn handle_events(&mut self) {
        while let Some(Event { id, event, time }) = self.gilrs.next_event() {
            match event.clone() {
                gilrs::ev::EventType::AxisChanged(axis, _, _) => {
                    match axis {
                        gilrs::Axis::LeftStickY => {
                            self.handle_lj_event(self.gilrs.gamepad(id).value(axis));
                        },
                        _ => {}
                    }
                },
                gilrs::ev::EventType::ButtonChanged(button, value, _) => {
                    match button {
                        Button::LeftTrigger2 => {
                            self.handle_lt_event(value);
                        },
                        Button::RightTrigger2 => {
                            self.handle_rt_event(value);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }

    fn handle_lt_event(&mut self, value: f32) {
        //println!("Handling lt event: {}", value);
        self.lt_state.value = value;
    }

    fn handle_rt_event(&mut self, value: f32) {
        //println!("Handling rt event: {}", value);
        self.rt_state.value = value;
    }

    fn handle_lj_event(&mut self, value: f32) {
        //println!("Handling lj event: {}", value);

        if value.abs() < self.joystick_deadzone {
            self.left_joystick_state.y = 0.0;
            return;
        } else {
            self.left_joystick_state.y = value;
        }
    }
}