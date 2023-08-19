use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

extern crate abi_stable;
extern crate image;
extern crate rusttype;

// use abi_stable::{library::RootModule, declare_root_module_statics, package_version_strings};
use image::imageops;

///requests that are sent from the controller to the pluginc
#[derive(Serialize, Deserialize)]
pub enum ServerToClientMessage {
    PRESSED(u8),
    ASSETCALL(u8),
    NEWBUTTON(NewButton),
    ERROR,
}

#[derive(Serialize, Deserialize)]
pub enum ClientToServerMessage {
    INITIALREPORT(InitialReport),
    ASSETREPORT(AssetReport),
    ERROR,
}

#[derive(Serialize, Deserialize)]
pub enum Error {
    UNEXPECTED_MESSAGE,
}

#[derive(Serialize, Deserialize)]
pub struct InitialReport {
    pub name: String,
    pub desc: Option<String>,
    pub author: Option<String>,
    pub buttons: Vec<String>,
}

///will be sent from the controller to the plugin to register a new button
#[derive(Serialize, Deserialize)]
pub struct NewButton {
    id: u8,
    position: [u8; 2],
}

///reports data such as the icon
#[derive(Serialize, Deserialize)]
pub struct AssetReport {
    id: u8,
    text: String,
    rgb: [u8; 3],
    //will be encoded in base64 for easier transport over json.
    image: String,
}

//default icon loader. will support svg, png, jpeg, and whatever else is supported by the image
//library
pub fn load_icon(path: PathBuf) -> Option<image::DynamicImage> {
    let svg = &OsString::from("svg");

    match path.extension().unwrap() {
        svg => {
            todo!()
        }
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
        _ => None,
    }
}

fn log_info() {
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
