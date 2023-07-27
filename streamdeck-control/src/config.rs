use std::{path::{Path, PathBuf}, fs::File, io::Read, process::Command, ffi::{OsString, OsStr}, collections::HashMap};
use Rstreamdeck_lib::Button;
use image::imageops;
use serde::Deserialize;
use anyhow::{Result, Ok};


pub fn load_config(path: Option<String>) -> Result<Deckstate> {
    if let Some(path) = path {
        let path = PathBuf::from(path);
        return Ok(Deckstate::new(path));
    }

    auto_load_config()
}

//will also create config if it does not exist
//
#[cfg(target_os = "linux")]
fn auto_load_config() -> Result<Deckstate> {
    use std::io::Write;

    let path = "config.toml";

    //why is this a struct
    let dirs = xdg::BaseDirectories::with_prefix("rstreamdeck").expect("dirs could not be loaded");

    match dirs.find_config_file(path) {
        //TODO: add error handling to deckstate new

        Some(file) => {
            Ok(Deckstate::new(file))
        },
        None => {
            let new_path = dirs.place_config_file(path).expect("unable to create config file");
            let mut new_file = File::create(new_path.clone())?;

            new_file.write_all(b"[deck]\nbrightness = 100\n\n#basic button layout.\n[b1]\n#text = <string goes here>\n#icon = <path goes here>\nrgb = [0,0,255]\n#command = <insert command here> ")?;
            drop(new_file);

            Ok(Deckstate::new(new_path))
        }
    }
}

#[cfg(target_os = "windows")]
fn auto_load_config() -> Result<Deckstate> {
    unimplemented!();
}


//#[derive(Debug)]
pub struct Deckstate {
    pub brightness: u8,
    pub buttons: Vec<Box<dyn Rstreamdeck_lib::Button>>
}

impl Deckstate {
    pub fn new(path: PathBuf) -> Self {
        let base = stdStreamdeckToml::new(path);
        let config = base.into();
        config
    }
}

#[derive(Debug)]
pub struct DButton {
    pub text: Option<String>,
    pub icon: Option<image::DynamicImage>,
    pub rgb: Option<[u8; 3]>,
    pub command: Option<Command>
}

impl Rstreamdeck_lib::Button for DButton {
    fn new(text: Option<String>, icon: Option<PathBuf>, rgb: Option<[u8; 3]>, opts: Option<HashMap<String, String>>) -> Box<Self> { 

        let comm = match opts {
            Some(c) => {
                match c.contains_key("command") {
                    true => c.get("command"),
                    false => None 
                }
            }
            None => None
        };



        Box::new(DButton {
            text: text,
            icon: match icon {
                Some(i) => Rstreamdeck_lib::load_icon(i),
                None => None
            },
            rgb: rgb,
            command: todo!(), 

        })
    }

    fn pressed(&mut self) -> Result<Option<Rstreamdeck_lib::ButtonRequest>, ()> {
        unimplemented!() 
    }

    fn get_icon(&mut self) -> Option<image::DynamicImage> {
        self.icon
    }

    fn get_rgb(&mut self) -> Option<[u8; 3]> {
        self.rgb
    }

    fn get_text(&mut self) -> Option<String> {
        self.text
    }
}
 


#[derive(Deserialize, Debug, Clone)]
pub enum DeckType {
    StreamdeckOriginal,
    StreamDeckOriginalV2,
    StreamDeckV2,
    StreamDeckXL,
    StreamDeckMini
}

#[derive(Clone, Debug, Deserialize)]
pub struct ButtonConfig {
    pub text: Option<String>,
    pub icon: Option<PathBuf>,
    pub rgb: Option<[u8; 3]>,
    pub opts: Option<HashMap<String, String>>,
    pub Button: Option<String>,
    pub plugin: Option<String>
}

#[derive(Deserialize, Debug)]
struct DeckConfig {
    brightness: Option<u8>
}

//I am sorry to future me if this needs to be redone. update: I have already had to redo it once
#[derive(Debug, Deserialize)]
struct stdStreamdeckToml {
    deck: DeckConfig,

    b1: Option<ButtonConfig>,
    b2: Option<ButtonConfig>,
    b3: Option<ButtonConfig>,
    b4: Option<ButtonConfig>,
    b5: Option<ButtonConfig>,
    b6: Option<ButtonConfig>,
    b7: Option<ButtonConfig>,
    b8: Option<ButtonConfig>,
    b9: Option<ButtonConfig>,
    b10: Option<ButtonConfig>,
    b11: Option<ButtonConfig>,
    b12: Option<ButtonConfig>,
    b13: Option<ButtonConfig>,
    b14: Option<ButtonConfig>,
    b15: Option<ButtonConfig>
}

impl stdStreamdeckToml {
    fn new(path: PathBuf) -> stdStreamdeckToml {
        let mut file = File::open(path).unwrap();
        let mut content = String::new();

        file.read_to_string(&mut content).unwrap();

        toml::from_str(content.as_str()).unwrap()
    }
}

impl Into<Deckstate> for stdStreamdeckToml {
    fn into(self) -> Deckstate {
         let button_vec = vec![self.b1, self.b2, self.b3, self.b4, self.b5, self.b6, self.b7, self.b8, self.b9, self.b10, self.b11, self.b12, self.b13, self.b14, self.b15];

        let mut buttons: Vec<Box<dyn Rstreamdeck_lib::Button>> = Vec::new();

        for i in (0..15).into_iter().zip(button_vec) {
            let (index, temp) = i;
           
            let config = match temp {
                Some(con) => con,
                None => {
                    buttons.push(Box::new(DButton{text: None, icon: None, rgb: None, command: None}));
                    eprintln!("button missing");
                    continue;
                }
            };


            buttons.push(DButton::new(index, config.text, config.icon, config.rgb, config.opts));
        }

        Deckstate { 
            brightness: self.deck.brightness.unwrap_or(100 as u8), 
            buttons: buttons 
        }
    }
}
