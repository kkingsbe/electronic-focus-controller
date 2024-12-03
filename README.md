# Electronic Focus Controller

This project is a Rust-based application designed to control a custom-made electronic focus adjustment module for a telescope. using various input methods such as gamepads and knobs. The application provides a graphical user interface (GUI) using the `egui` crate.

## Features

- **Gamepad Control**: Use a gamepad to control the focus device. The gamepad driver handles input events and calculates speed and setpoint for the focus device. This allows for a relatively high level of precision.
- **Knob Control**: You can also use a physical usb volume knob to adjust the focus. The knob can toggle between setpoint and speed control modes.

## Project Structure

- `src/focus_controller.rs`: Contains the `FocusController` struct, which manages the focus device's state and communication.
- `src/gamepad.rs`: Implements the `GamepadDriver` and handles gamepad input events.
- `src/knob.rs`: Implements the `Knob` struct for handling knob input and control modes.
- `src/main.rs`: Entry point of the application, initializes the GUI and handles user interactions.
- `src/windows_volume_controller.rs`: Provides an interface to control Windows system volume.

## Contact

For questions or support, please contact me at kkingsbe@gmail.com

