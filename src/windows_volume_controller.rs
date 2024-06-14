use windows_volume_control::AudioController;

pub struct WindowsVolumeController {
    controller: AudioController
}

impl WindowsVolumeController {
    pub fn new() -> WindowsVolumeController {
        unsafe {
            return WindowsVolumeController {
                controller: AudioController::init(None)
            }
        }
    }

    pub fn read_volume(&self) -> f32 {
        unsafe {
            let master_session = self.controller.get_session_by_name("master".to_string());
            println!("{:#?}", master_session.unwrap().getVolume());

            return master_session.unwrap().getVolume();
        }
    }

    pub fn set_volume(&self, volume: f32) -> () {
        if(volume > 1.0 || volume < 0.0) {
            panic!("Volume must be between 0.0 and 1.0");
        }
        
        unsafe {
            let master_session = self.controller.get_session_by_name("master".to_string());
            master_session.unwrap().setVolume(volume);
        }
    }

    pub fn init(&mut self) {
        unsafe {
            self.controller.GetSessions();
            self.controller.GetDefaultAudioEnpointVolumeControl();
            self.controller.GetAllProcessSessions();
            self.controller.get_all_session_names();
        }
    }
}