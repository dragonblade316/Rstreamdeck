use base64::Engine;
use clap::Parser;
//cuse anyhow::Result;
use core::panic;
use image::DynamicImage;
use std::io::{Cursor, Error};
use std::io::{Read, Write};
use std::time::Duration;
use std::{collections::HashMap, fmt::Result, os::unix::net::UnixStream, todo};
use Rstreamdeck_lib::{ButtonReport, ClientToServerMessage, InitialReport, ServerToClientMessage };

pub use Rstreamdeck_lib::ButtonDesc;



pub struct Context {
    socket: UnixStream,
}

impl Context {
    fn new() -> Self {
        let socket = UnixStream::connect("/tmp/rdeck.sock").unwrap();

        Context {
            socket
        }
    }

    pub fn send_text(&mut self, index: u8, text: &str, fontsize: Option<f32>, text_offset: Option<(i32, i32)>) {
        println!("sending text: {text}");
        self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
            id: index,
            text: Some(text.to_string()),
            fontsize,
            text_offset,
            image: None,
            rgb: None,
        }));
    }

    pub fn send_image(&mut self, index: u8, image: DynamicImage) {
        let b64 = base64::engine::general_purpose::STANDARD_NO_PAD;

        let mut buf: Vec<u8> = Vec::new();
        &image.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png);

        let mut serialized = String::new();
        b64.encode_string(buf, &mut serialized);


        self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
            id: index as u8,
            text: None,
            fontsize: None,
            text_offset: None,
            image: Some(serialized),
            rgb: None,
        }));
    }

    pub fn send_rgb(&mut self, index: u8, rgb: [u8; 3]) {
        self.send_message(ClientToServerMessage::BUTTONREPORT(ButtonReport {
            id: index,
            text: None,
            fontsize: None,
            text_offset: None,
            image: None,
            rgb: Some(rgb),
        }));
    }


    fn send_message(&mut self, message: ClientToServerMessage) {
        let json = serde_json::to_string(&message).unwrap();
        Rstreamdeck_lib::write_string_to_rdeck_socket(&mut self.socket, json);
    }
}

#[derive(Debug)]
pub enum ButtonEvent {
    Pressed,
    Depressed,
}

//#[derive(Debug)]
pub struct PluginAPI<CBE: FnMut(&mut Context, u8, Option<String>, [u8; 2], Option<HashMap<String, String>>), BE: FnMut(&mut Context, ButtonEvent, u8)> {
    context: Context,

    name: String,
    desc: Option<String>,
    author: Option<String>,

    buttons: Vec<ButtonDesc>,
    profiles: HashMap<String, String>,

    new_button_function: CBE, 


    button_callback: BE,
}

impl<CBE, BE> PluginAPI<CBE, BE> 
where CBE: FnMut(&mut Context, u8, Option<String>, [u8; 2], Option<HashMap<String, String>>), BE: FnMut(&mut Context, ButtonEvent, u8) {
    ///starts the plugin api and sends the initial report to Rstreamdeck. This should be the first thing done to
    ///ensure the plugin manager does not timeout and kill the plugin
    pub fn new(
        name: &str,
        desc: Option<&str>,
        author: Option<&str>,

        buttons: Vec<ButtonDesc>,
        profiles: HashMap<String, String>, 

        new_button_function: CBE, 

        button_callback: BE,
        
    ) -> anyhow::Result<Self> {

        let context = Context::new();

        //no idea if match is the right thing to use here. idk
        let desc = match desc {
            Some(i) => Some(i.to_string()),
            None => None,
        };

        let author = match author {
            Some(i) => Some(i.to_string()),
            None => None,
        };

        let mut papi = Self {
            context,
            name: name.to_string(),
            desc,
            author,
            buttons,
            profiles,
            new_button_function,
            button_callback,
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
        self.context.send_message(ClientToServerMessage::INITIALREPORT(InitialReport {
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

    pub fn set_blocking(&mut self, set: bool){
        self.context.socket.set_nonblocking(set);
    }

    pub fn get_ctx(&mut self) -> &mut Context {
        &mut self.context
    }

    //this will later be moved to another thread. I just need proof of concept
    pub fn update(&mut self) {
        // self.context.socket.set_read_timeout(Some(Duration::new(1, 0)));
        fn send_error(socket: &mut UnixStream) {
            Rstreamdeck_lib::write_string_to_rdeck_socket(socket, serde_json::to_string(&Rstreamdeck_lib::ClientToServerMessage::ERROR("bad message".to_string())).unwrap());
        }


            let json = match Rstreamdeck_lib::recive_message_from_server(&mut self.context.socket) {
                Ok(k) => match k {
                    Some(i) => i,
                    None => {
                        // print!("no value. aborting update");
                        return
                    }
                },
                Err(e) => {
                    //std::io::Error
                    ServerToClientMessage::Error(Rstreamdeck_lib::ServerError::OTHER)
                },
            };

            println!("{json:?}");

            match json {
                ServerToClientMessage::PRESSED(id) => {
                    println!("pressed :)");
                    (self.button_callback)(&mut self.context, ButtonEvent::Pressed, id);
                },
                ServerToClientMessage::DEPRESSED(id) => {
                    println!("depressed :(");
                    (self.button_callback)(&mut self.context, ButtonEvent::Depressed, id);
                },
                ServerToClientMessage::NEWBUTTON(id) => {
                    println!("got new button request");
                    (self.new_button_function)(&mut self.context, id.id, id.button, id.position, id.opts);
                },
                _ => {
                    println!("sending error");
                    send_error(&mut self.context.socket)
                }
            };
    }
}


#[derive(Parser)]
struct PluginArgs {
    help: bool
}
