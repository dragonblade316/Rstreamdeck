use crate::hardware::Protocol;
use std::{collections::HashMap, format, process::Command, unimplemented};

pub fn new_button_protocol(
    button: String,
    opts: Option<HashMap<String, String>>,
) -> Option<Box<dyn Protocol>> {
    match button.as_str() {
        "command" => Some(Box::new(CommandButton::new(opts))),
        _ => None,
    }
}

struct CommandButton {
    command: Command,
}

impl CommandButton {
    fn new(opts: Option<HashMap<String, String>>) -> Self {
        return Self {
            command: Command::new(opts.unwrap_or(HashMap::new()).get("command").unwrap_or(
                /*wth*/ &"echo yeah, you forgot to set the opts".to_string(),
            )),
        };
    }
}

impl Protocol for CommandButton {
    fn pressed(&mut self) {
        self.command.spawn();
    }
}

struct MultimediaButton {}

struct VolumeButton {}

struct not_used;

impl Protocol for not_used {
    fn pressed(&mut self) {
        return;
    }
}
