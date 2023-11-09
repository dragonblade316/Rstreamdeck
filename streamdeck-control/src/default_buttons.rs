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
    //TODO: use command output 
    command_output: Option<String>,
    display: Option<bool>,
    display_command: Option<Command>,
    
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
        //honestly this looks a little like garbage but whatever.
        return Self {
            command: Command::new(opts.clone().unwrap_or(HashMap::new()).get("command").unwrap_or(/*wth*/ &"echo yeah, you forgot to set the opts".to_string(),)),
            command_output: None,
            display: match opts.clone().unwrap_or(HashMap::new()).get("display") {
                Some(t) => Some(t.eq("true")),
                None => None,
            },
            display_command: match opts.unwrap_or(HashMap::new()).get("display_command") {
                Some(c) => Some(Command::new(c)),
                None => None
            }
        };
    }
}

impl Protocol for CommandButton {
    fn pressed(&mut self) {
        println!("command triggerd");
        self.command.spawn().unwrap();
    }
    //sorry to whoever has to read this mess
    fn get_text(&self) -> Option<String> {
        match self.display.unwrap_or(return None) {
            true => match self.display_command {
                Some(c) => match c.output() {
                    Ok(out) => String::from_utf8(out.stdout).ok(),
                    Err(e) => None,
                },
                //TODO: allow command output to be used
                _ => None,
            },
            _ => None,
        }         
    }
}

//TODO: figure out how the heck this is suppoesd to work.
struct ChangeProfileButton {
    profile: String,
}

//TODO: add a button for macros and complex keybindings. (most likely using vim keybind notation)

#[derive(Debug)]
struct not_used;

impl Protocol for not_used {
    fn pressed(&mut self) {
        return;
    }
}
