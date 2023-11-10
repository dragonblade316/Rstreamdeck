use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::File,
    io::Read,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
// use Rstreamdeck_lib::Button;
use crate::hardware::{Button, Deck};
use anyhow::{Ok, Result};
use image::imageops;
use serde::Deserialize;



//TODO: rewite this
pub fn load_deck_from_config(path: Option<String>) -> Result<Deck> {
    if let Some(path) = path {
        let path = PathBuf::from(path);
        return Deck::new(StreamdeckConfig::new(path).expect("failed to load config"));
    }

    auto_load_config()
}

//will also create config if it does not exist
//TODO: also rewrite this
#[cfg(target_os = "linux")]
fn auto_load_config() -> Result<Deck> {
    use std::io::Write;

    let path = "default.toml";

    //why is this a struct
    let dirs = xdg::BaseDirectories::with_prefix("rstreamdeck").expect("dirs could not be loaded");

    

    // match dirs.find_config_file(path) {
    //     //TODO: add error handling to deckstate new
    //     Some(file) => Deck::new(StreamdeckConfig::new(file).expect("failed to load config")),
    //     None => {
    //         let new_path = dirs
    //             .place_config_file(path)
    //             .expect("unable to create config file");
    //         let mut new_file = File::create(new_path.clone())?;
    //
    //         new_file.write_all(include_bytes!("../exampleconfig.toml"));
    //         drop(new_file);
    //
    //         Deck::new(StreamdeckConfig::new(new_path).expect("failed to load config"))
    //     }
    // }
    //
    Deck::new(StreamdeckConfig::new(dirs.get_config_home())?)
}


//#[derive(Debug)]

#[derive(Deserialize, Debug, Clone)]
pub enum DeckType {
    StreamdeckOriginal,
    StreamDeckOriginalV2,
    StreamDeckV2,
    StreamDeckXL,
    StreamDeckMini,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ButtonConfig {
    pub text: Option<String>,
    pub icon: Option<PathBuf>,
    pub rgb: Option<[u8; 3]>,
    pub opts: Option<HashMap<String, String>>,
    pub button: Option<String>,
    pub plugin: Option<String>,
}

//I am sorry to future me if this needs to be redone. update: I have as.insert(i.Plugin.new(plugin_socket, plugin_descriptor));lready had to redo it once
#[derive(Debug, Deserialize)]
pub struct StreamdeckProfileToml {
    pub brightness: Option<u8>,

    pub b1: Option<ButtonConfig>,
    pub b2: Option<ButtonConfig>,
    pub b3: Option<ButtonConfig>,
    pub b4: Option<ButtonConfig>,
    pub b5: Option<ButtonConfig>,
    pub b6: Option<ButtonConfig>,
    pub b7: Option<ButtonConfig>,
    pub b8: Option<ButtonConfig>,
    pub b9: Option<ButtonConfig>,
    pub b10: Option<ButtonConfig>,
    pub b11: Option<ButtonConfig>,
    pub b12: Option<ButtonConfig>,
    pub b13: Option<ButtonConfig>,
    pub b14: Option<ButtonConfig>,
    pub b15: Option<ButtonConfig>,
}

impl StreamdeckProfileToml {
    fn new(path: PathBuf) -> StreamdeckProfileToml {
        println!("Attempting to load toml file from {}", path.to_str().unwrap());

        let mut file = File::open(path).unwrap();
        let mut content = String::new();

        file.read_to_string(&mut content).unwrap();

        toml::from_str(content.as_str()).unwrap()
    }
}

///Mainly a logic container to the configuration loading logic
pub struct StreamdeckConfig {
    pub profiles: HashMap<String, StreamdeckProfileToml>,
}

impl StreamdeckConfig {


    fn new(path: PathBuf) -> Result<Self> {
        let mut map: HashMap<String, StreamdeckProfileToml> = HashMap::new();

        //default path
        let mut dpath = path.clone();
        dpath.push("default.toml");

        println!("does the config path exist {}", dpath.exists());

        if !dpath.exists() {
            let mut new_file = File::create(dpath.clone())?;

            new_file.write_all(include_bytes!("../exampleconfig.toml"));
            drop(new_file);
        }

        map.insert(format!("default"), StreamdeckProfileToml::new(dpath));

        //profile path
        let mut ppath = path.clone();
        ppath.push("plugins");

        match ppath.exists() && ppath.is_dir() {
            true => {
                for i in std::fs::read_dir(ppath)? {
                    unimplemented!();
                }
            },
            _ => {},
        }

        Ok(Self {
            profiles: map
        })
    }
}
