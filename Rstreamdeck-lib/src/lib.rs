use rusttype::Font;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::unimplemented;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

extern crate image;
extern crate rusttype;

// use abi_stable::{library::RootModule, declare_root_module_statics, package_version_strings};
use image::imageops;

//TODO: every request will have a uuid the responce must have the same uuid in order to allow for
//multiple simltainius requests. Hello I'm looking at this and realizing it does not matter.

///requests that are sent from the controller to the pluginc
#[derive(Serialize, Deserialize, Clone)]
pub enum ServerToClientMessage {
    PRESSED(u8),
    DEPRESSED(u8),
    ASSETCALL(u8),
    NEWBUTTON(NewButton),
    ERROR,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ClientToServerMessage {
    INITIALREPORT(InitialReport),
    BUTTONREPORT(ButtonReport),
    ERROR,
}

#[derive(Serialize, Deserialize)]
pub enum Error {
    UNEXPECTED_MESSAGE,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InitialReport {
    pub name: String,
    pub desc: Option<String>,
    pub author: Option<String>,
    pub buttons: Vec<ButtonDesc>,
    pub profiles: HashMap<String, String>,
}

///will be sent from the controller to the plugin to register a new button
#[derive(Serialize, Deserialize, Clone)]
pub struct NewButton {
    pub id: u8,
    pub button: Option<String>,
    pub position: [u8; 2],
    pub opts: Option<HashMap<String, String>>,
}


///reports data such as the icon
#[derive(Serialize, Deserialize, Clone)]
pub struct ButtonReport {
    pub id: u8,
    pub text: Option<String>,
    pub rgb: Option<[u8; 3]>,
    //will be encoded in base64 for easier transport over json.
    pub image: Option<String>,
}

///A descriptor for a button (Obviously)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ButtonDesc {
    name: String,
    desc: Option<String>,
    opts: Option<String>,
}

use std::io::{Read, Write};
///reads data from a socket to a string. this is ment to standardize the reading and writing of
///json between the plugin library and the application.
///technically this works on anything that implments the Read trait
pub fn read_string_from_rdeck_socket<T: Read>(socket: &mut T) -> anyhow::Result<String> {
    let mut len_buf: [u8; 8] = [0; 8];
    socket.read_exact(&mut len_buf);
    let mut buf: Vec<u8> = Vec::new();
    buf.resize(u64::from_le_bytes(len_buf) as usize, 0);
    socket.read_exact(&mut buf);
    Ok(String::from_utf8(buf).unwrap())
}

pub fn write_string_to_rdeck_socket<T: Write>(socket: &mut T, json: String) {
    let buf = json.into_bytes();
    let size = buf.len();
    let _ = socket.write(&(size as u64).to_le_bytes());
    let _ = socket.write(&buf);
}

//default icon loader. will support svg, png, jpeg, and whatever else is supported by the image
//library
//INFO: this should probably be a result since the image could either not be present or unreadable
pub fn load_icon(path: PathBuf) -> Option<image::DynamicImage> {
    let svg = &OsString::from("svg");

    match path.extension().unwrap() {
        //svg => None,
        _ => match image::io::Reader::open(path) {
            Ok(reader) => match reader.decode() {
                Ok(image) => Some(image.resize_exact(72, 72, imageops::FilterType::Gaussian)),
                Err(e) => {
                    eprintln!("{:?}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("{:?}", e);
                None
            }
        },
    }
}

fn log_info(info: String) {
    unimplemented!()
}

fn log_warning() {
    unimplemented!()
}

fn log_error() {
    unimplemented!()
}

// //here if you need it
// fn load_font(path: PathBuf) -> Option<'static rusttype::Font> {
//     unimplemented!()
// }

// //will attempt to load a system font
// fn load_system_font(font: String) -> Option<rusttype::Font> {
//     unimplemented!()
// }
