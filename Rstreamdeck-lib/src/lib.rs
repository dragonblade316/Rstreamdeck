use std::{path::PathBuf, ffi::{OsString, OsStr}};
use std::collections::HashMap;

extern crate image;
extern crate rusttype;
extern crate abi_stable;

// use abi_stable::{library::RootModule, declare_root_module_statics, package_version_strings};
use image::imageops;

use abi_stable::{
    declare_root_module_statics,
    erased_types::{DeserializeDyn, IteratorItem, SerializeProxyType},
    external_types::{RawValueBox, RawValueRef},
    library::{LibraryError, RootModule},
    package_version_strings,
    sabi_types::{RMut, VersionStrings},
    std_types::{RArc, RBox, RBoxError, RCowStr, RResult, RStr, RString},
    DynTrait, StableAbi,
};

pub enum ButtonRequest {
    REQUESTCONTROL,
}


pub trait Button {

    ///unforturetly your new function may get broken every now and again
    fn new(text: Option<String>, icon: Option<PathBuf>, rgb: Option<[u8; 3]>, opts: Option<HashMap<String, String>>) -> Box<Self> where Self: Sized;

    ///Will trigger when a button press is detected. If you run into an error please to not use
    ///unwrap as that will crash the whole system. instead just return Err() 
    fn pressed(&mut self) -> Result<Option<ButtonRequest>, ()>;

    ///please optimize to return a 72x72 image as this function will be called quite often and
    ///resizing every call could cause a performance hit
    fn get_icon(&mut self) -> Option<image::DynamicImage>;

    ///the default is black
    fn get_rgb(&mut self) -> Option<[u8; 3]>;

    ///the text will be displayed on the 
    fn get_text(&mut self) -> Option<String>;
}
//
// pub trait Plugin: RootModule {
//     fn init();
//
//     fn help(button: Option<String>) -> String;
// }

type button_init = dyn Fn(Option<String>, Option<PathBuf>, Option<[u8; 3]>, Option<HashMap<String, String>>) -> Box<dyn Button>;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct Plugin {
    help_text: String, 
    button_inits: HashMap<String, Box<button_init>>
}

impl RootModule for Plugin_Ref {
    declare_root_module_statics! { Plugin_Ref }
    const BASE_NAME: &'static str = "button_plugin";
    const NAME: &'static str = "button_plugin";
    const VERSION_STRINGS: abi_stable::sabi_types::VersionStrings = package_version_strings!();

    fn initialization(self) -> Result<Self, abi_stable::library::LibraryError> {
        Ok(self)
    }
}

    
//default icon loader. will support svg, png, jpeg, and whatever else is supported by the image
//library
pub fn load_icon(path: PathBuf) -> Option<image::DynamicImage> {
    
    let svg = &OsString::from("svg");


    match path.extension().unwrap() {
        svg => {
            todo!()
        },
        _ => match image::io::Reader::open(path) {
            Ok(reader) => {
                match reader.decode() {
                    Ok(image) => Some(image.resize_exact(72, 72, imageops::FilterType::Gaussian)),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                None
            }
        },
        _ => None
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
