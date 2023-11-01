use crate::hardware::Protocol;
use std::{collections::HashMap, fmt::Debug, format, println, process::Command, unimplemented};

pub fn new_button_protocol(
    button: String,
    opts: Option<HashMap<String, String>>,
) -> Option<Box<dyn Protocol>> {
    println!("button is {:?}", button);
    match button.as_str() {
        "command" => return Some(Box::new(CommandButton::new(opts))),
        _ => None,
    }
}

#[derive(Debug)]
struct CommandButton {
    command: Command,
}

impl CommandButton {
    fn new(opts: Option<HashMap<String, String>>) -> Self {
        println!("building command button");

        println!(
            "command is {:?}",
            opts.as_ref()
                .unwrap_or(&HashMap::new())
                .get("command")
                .unwrap_or(/*wth*/ &"echo yeah, you forgot to set the opts".to_string(),)
        );
        return Self {
            command: Command::new(opts.unwrap_or(HashMap::new()).get("command").unwrap_or(
                /*wth*/ &"echo yeah, you forgot to set the opts".to_string(),
            )),
        };
    }
}

impl Protocol for CommandButton {
    fn pressed(&mut self) {
        println!("command triggerd");
        self.command.spawn().unwrap();
    }
}

struct ChangeProfileButton {}

#[derive(Debug)]
struct not_used;

impl Protocol for not_used {
    fn pressed(&mut self) {
        return;
    }
}
