use crate::hardware::Protocol;
use std::{collections::HashMap, fmt::Debug, format, println, process::Command, unimplemented, sync::{Mutex, Arc}};
use Rstreamdeck_lib::Instruction;

pub fn new_button_protocol(
    button: String,
    opts: Option<HashMap<String, String>>,
) -> Option<Box<dyn Protocol>> {
    println!("button is {:?}", button);
    match button.as_str() {
        "command" => return Some(Box::new(CommandButton::new(opts))),
        "changeProfileButton" => return Some(Box::new(ChangeProfileButton::new(opts))),
        _ => None,
    }
}

#[derive(Debug)]
struct CommandButton {
    command: Command,
    //TODO: use command output 
    command_output: Arc<Mutex<Option<String>>>,

    //TODO: we can use a seperate thread to keep up with polling. this will prevent me from
    //having to add an update method to Protocal.
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

        //idk why this needs to be three vars but it made the borrow checker happy
        let e = "echo yeah, you forgot to set the opts".to_string(); 
        let opts = opts.clone().unwrap_or(HashMap::new());
        let base_str = opts.get("command").unwrap_or(&e);

        let mut items = base_str.split_whitespace().map(|i| i.to_string()).collect::<Vec<String>>();
        //dont need to have a seperate args var since items would only contain arguments after
        //removing 0
        let comm = items.remove(0);


        let mut command = Command::new(comm);
        command.args(items);

        

        //honestly this looks a little like garbage but whatever.
        return Self {
            command, 
            command_output: Arc::new(Mutex::new(None)),
            display: match opts.get("display") {
                Some(t) => Some(t.eq("true")),
                None => None,
            },
            display_command: match opts.get("display_command") {
                Some(c) => Some(Command::new(c)),
                None => None
            }
        };
    }
}

impl Protocol for CommandButton {
    fn pressed(&mut self) {
        println!("command triggerd");
        let t = self.command.spawn();
        match t {
            Err(i) => error!("Command failed with {i}"),
            _ => {}
        }
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
#[derive(Debug)]
struct ChangeProfileButton {
    profile: String,
    switch: bool,
}

impl ChangeProfileButton {
    fn new(opts: Option<HashMap<String, String>>) -> Self{

        opts.as_ref().unwrap().get("profile").unwrap();
        ChangeProfileButton { profile: opts.as_ref().unwrap().get("profile").unwrap().to_string(), switch: false }
    }
}

impl Protocol for ChangeProfileButton {
    fn pressed(&mut self) {
        self.switch = true;
    } 
    fn get_instruction_request(&mut self) -> Option<Instruction> {
        match self.switch {
            true => {
                self.switch = false;
                Some(Instruction::ChangeProfile(self.profile.to_owned()))
            },
            false => None
        }
    }
}

//TODO: add a button for macros and complex keybindings. (most likely using vim keybind notation)

#[derive(Debug)]
struct NotUsed;

impl Protocol for NotUsed {
    fn pressed(&mut self) {
        return;
    }
}
