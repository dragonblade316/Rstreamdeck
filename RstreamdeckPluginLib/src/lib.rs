use base64::Engine;
use clap::Parser;
//cuse anyhow::Result;
use core::panic;
use image::DynamicImage;
use std::io::Cursor;
use std::io::{Read, Write};
use std::time::Duration;
use std::{collections::HashMap, fmt::Result, os::unix::net::UnixStream, todo};
use Rstreamdeck_lib::{ButtonReport, ClientToServerMessage, InitialReport, ServerToClientMessage, ButtonDesc};


//#[derive(Debug)]
pub struct PluginAPI<F, NBF> 
where F: FnMut(&mut PluginAPI<F, NBF>, u8), NBF: FnMut(&mut PluginAPI<F, NBF>, u8, Option<String>, [u8; 2], Option<HashMap<String, String>>)
{
    socket: UnixStream,
    name: String,
    desc: Option<String>,
    author: Option<String>,

    buttons: Vec<ButtonDesc>,
    profiles: HashMap<String, String>,

    new_button_function: NBF,

    pressed_function: F,
    depressed_function: F,
 
}

impl<F, NBF> PluginAPI<F, NBF> 
where F: FnMut(&mut PluginAPI<F, NBF>, u8), NBF: FnMut(&mut PluginAPI<F, NBF>, u8, Option<String>, [u8; 2], Option<HashMap<String, String>>)
{
    ///starts the plugin api and sends the initial report this should be the first thing done to
    ///ensure the plugin manager does not dismiss and kill the plugin
    pub fn new(
        name: String,
        desc: Option<String>,
        author: Option<String>,

        buttons: Vec<ButtonDesc>,
        profiles: HashMap<String, String>, 

        new_button_function: NBF,

        pressed_function: F,
        depressed_function: F,
        
    ) -> anyhow::Result<Self> {
        let socket = UnixStream::connect("/tmp/rdeck.sock").unwrap();

        let mut papi = Self {
            socket,
            name,
            desc,
            author,
            buttons,
            profiles,
            new_button_function,
            pressed_function,
            depressed_function,
        };

        papi.send_initial_report();

        Ok(papi)
    }

    //TODO: may want to replace the use of clone here. then again there would be almost no benifit
    fn send_initial_report(&mut self) {
        let thing = &self;
        // let buttons = self
        //     .buttons
        //     .clone()
        //     .into_iter()
        //     .map(|e| {
        //         let (s, _) = e;
        //         s
        //     })
        //     .collect::<Vec<String>>();
        //
        self.send_message(ClientToServerMessage::INITIALREPORT(InitialReport {
            name: self.name.clone(),
            desc: self.desc.clone(),
            author: self.author.clone(),
            buttons: self.buttons.clone(),
            profiles: self.profiles.clone(),

        }));
    }

    // //TODO: sending the image every update could be bad for perfomance. may be able to use Deref
    // //and DerefMut to detect changes
    // fn send_button_report(&mut self, index: u32) {
    //     let button = &self.active_buttons.get(&(index as u8)).unwrap();
    //
    //     let b64 = base64::engine::general_purpose::STANDARD_NO_PAD;
    //
    //     let imstr = match button.get_image() {
    //         Some(i) => {
    //             let mut buf: Vec<u8> = Vec::new();
    //             &i.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png);
    //
    //             let mut serialized = String::new();
    //             b64.encode_string(buf, &mut serialized);
    //             Some(serialized)
    //         }
    //         None => None,
    //     };
    //
    //     self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
    //         id: index as u8,
    //         text: button.get_text(),
    //         image: imstr, //base64,
    //         rgb: button.get_rgb(),
    //     }));
    // }

    pub fn send_text(&mut self, index: u32, text: String) {
        self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
            id: index as u8,
            text: Some(text),
            image: None,
            rgb: None,
        }));
    }

    pub fn send_image(&mut self, index: u32, image: DynamicImage) {
        let b64 = base64::engine::general_purpose::STANDARD_NO_PAD;

        let mut buf: Vec<u8> = Vec::new();
        &image.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png);

        let mut serialized = String::new();
        b64.encode_string(buf, &mut serialized);


        self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
            id: index as u8,
            text: None,
            image: Some(serialized),
            rgb: None,
        }));
    }

    pub fn send_rgb(&mut self, index: u32, rgb: [u8; 3]) {
        self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
            id: index as u8,
            text: None,
            image: None,
            rgb: Some(rgb),
        }));
    }


    fn send_message(&mut self, message: ClientToServerMessage) {
        let json = serde_json::to_string(&message).unwrap();
        Rstreamdeck_lib::write_string_to_rdeck_socket(&mut self.socket, json);
    }

    //this will later be moved to another thread. I just need proof of concept
    pub fn update(&mut self) {
        self.socket.set_read_timeout(Some(Duration::new(1, 0)));
        loop {
            let message = Rstreamdeck_lib::read_string_from_rdeck_socket(&mut self.socket);

            match message {
                Ok(item) => {
                    let json = serde_json::from_str::<ServerToClientMessage>(&item.as_str())
                        .unwrap_or(continue);
                    match json {
                        ServerToClientMessage::PRESSED(id) => {
                            (self.pressed_function)(&mut self, id);
                        },
                        ServerToClientMessage::DEPRESSED(id) => {
                            (self.depressed_function)(&mut self, id);
                        }
                        ServerToClientMessage::NEWBUTTON(id) => {
                            (self.new_button_function)(&mut self, id.id, id.button, id.position, id.opts);
                        }
                        _ => {}
                    };
                }
                Err(e) => continue,
            }
        }
    }
}


#[derive(Parser)]
struct PluginArgs {
    help: bool
}
