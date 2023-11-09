use std::collections::HashMap;
use std::eprintln;
use std::fmt::Debug;
use std::println;
use std::time::Duration;
use rusttype::Font::*;

use crate::default_buttons::new_button_protocol;
use crate::plugin;
use anyhow::Result;
use image::DynamicImage;
use streamdeck::Colour;
use streamdeck::StreamDeck;

use crate::config::{StreamdeckProfileToml, StreamdeckConfig};
use crate::plugin::PluginManager;
//#[derive(Debug)]
pub struct Deck {
    deck: StreamDeck,
    // brightness: u8,
    // buttons: Vec<Button>,

    profiles: HashMap<String, Profile>,
    current_profile: String,
    manager: PluginManager,
    last_state: Vec<u8>,
}

impl Deck {
    pub fn new(mut config: StreamdeckConfig) -> Result<Deck> {
        let mut man = PluginManager::new().expect("failed to start plugin manager");

        let vendorid = 0x0FD9;
        let mut pid: Option<u16> = None;

        let hid = hidapi::HidApi::new().expect("could not open hidapi");

        hid.device_list().for_each(|i| {
            if i.vendor_id() == vendorid {
                pid = Some(i.product_id());
            }
        });

        let deck: StreamDeck;
        match pid {
            Some(id) => {
                deck = streamdeck::StreamDeck::connect_with_hid(&hid, vendorid, id, None)
                    .expect("could not connect to streamdeck");
            }
            None => panic!("no streamdeck detected"),
        };
        

        let mut profiles: HashMap<String, Profile> = HashMap::new();

        //The reason drain is used here is because it effectively uses HashMap::remove() which
        //returns an the actual objects instead of references that HashMap::get() returns. This is
        //fine because config.profiles is consumed here and not used further. this could be an
        //issue though 
        for i in config.profiles.drain() {
            let (name, profile) = i;

                
            let button_vec = vec![
                profile.b1, profile.b2, profile.b3, profile.b4, profile.b5, profile.b6, profile.b7, profile.b8,
                profile.b9, profile.b10, profile.b11, profile.b12, profile.b13, profile.b14, profile.b15,
            ];

            let mut buttons: Vec<crate::hardware::Button> = Vec::new();

            for i in (0..15).into_iter().zip(button_vec) {
                let (index, temp) = i;

                let bconfig = match temp {
                    Some(con) => con,
                    None => {
                        buttons.push(Button::empty());
                        eprintln!("button missing");
                        continue;
                    }
                };

                let plugin: Option<&plugin::Plugin>;

                let button = match bconfig.plugin {
                    Some(p) => Some(
                        man.get_button(index, p, bconfig.button, bconfig.opts)
                            .unwrap(),
                    ),
                    None => match bconfig.button {
                        Some(b) => new_button_protocol(b, bconfig.opts),
                        None => None,
                    },
                };

                let image = match bconfig.icon {
                    Some(t) => Rstreamdeck_lib::load_icon(t),
                    None => None,
                };

                buttons.push(Button::new(bconfig.text, bconfig.rgb, image, button));
            }

            profiles.insert(/*I can not beleve I am setting I am using to_string() on &string*/ name.to_string(), Profile {
                brightness: profile.brightness.unwrap_or(100),
                buttons,
            });
        }


        println!("deck creation successful");


        man.lock();

        //TODO: switch to profiles 
        Ok(Deck {
            deck,
            profiles,
            current_profile: format!("default"),
            manager: man,
            last_state: Vec::new(),
        })
    }

    fn change_profile(&mut self, profile: String) {
        if self.profiles.contains_key(&profile) {
            self.current_profile = profile;
        }
    }

    ///a helper function for getting the current profile
    fn get_current_profile(&self) -> &Profile {
        self.profiles.get(&self.current_profile).unwrap_or(self.profiles.get(&format!("default")).expect("default profile broken. Pls send help"))
    }

    // ///a helper function for getting the current profile
    // fn get_current_profile_mut(&mut self) -> &mut Profile {
    //     self.profiles.get_mut(&self.current_profile).unwrap_or(self.profiles.get_mut(&format!("default")).expect("default profile broken. Pls send help"))
    // }

    pub fn update(&mut self) {
        let mut status: Vec<ButtonStatus> = Vec::new();
        #[derive(Debug)]
        enum ButtonStatus {
            Pressed,
            Unchanged,
            Depressed,
        }

        let state = self
            .deck
            .read_buttons(Some(Duration::new(1, 0)))
            .unwrap_or(vec![0; 64]);

        //this determines which buttons have been pressed since the last interation of the loop.
        //this is needed so a command is not triggerd 100 time per second
        //Im now realizing this could be totally useless
        for i in state.iter().zip(&mut self.last_state) {
            let (is, was) = i;

            if is > was {
                status.push(ButtonStatus::Pressed);
                println!("pressed")
            } else if is < was {
                status.push(ButtonStatus::Depressed)
            } else {
                status.push(ButtonStatus::Unchanged)
            }
        }
        self.last_state = state;

        //SCREW THE STUPID BORROW CHECKER
        // let profile = self.get_current_profile_mut();
       
        //this is what I do to satisfy the borrow checker
        let profile_entries = self.profiles.keys().map(|x| x.clone()).collect::<Vec<String>>().clone();

        //I dont entirely know why this works instead of the above but it does not throw errors so
        //I'm not touching it
        let profile = self.profiles.get_mut(&self.current_profile).unwrap();

        for i in (0..self.deck.kind().keys())
            .zip(&mut profile.buttons)
            .zip(status)
        {
            let ((index, button), is_pressed) = i;

            self.deck.set_brightness(profile.brightness.clone());

            //yes I see I'm using a match patteren for only one thing. Deal with it. 
            match button.update() { 
                Some(i) => match i {
                    Instruction::ChangeProfile(p) => {
                        //and I have no idea why this works
                        if profile_entries.contains(&&p) {
                            self.current_profile = p;
                        }
                    },
                    //in case I ever miss something
                    _ => {}
                },
                _ => {}
            }

            //I'm now realizing that if transparent images are supported this code would not do as
            //intended. Eh I will deal with it if it becomes a problem
            match &button.image {
                Some(im) => self
                    .deck
                    .set_button_image(index, im.clone())
                    .expect("could nto set image"),
                None => match button.rgb {
                    Some(rgb) => self
                        .deck
                        .set_button_rgb(
                            index,
                            &Colour {
                                r: rgb[0],
                                g: rgb[1],
                                b: rgb[2],
                            },
                        )
                        .expect("could not set rgb"),
                    None => self
                        .deck
                        .set_button_rgb(index, &Colour { r: 0, g: 0, b: 0 })
                        .expect("could not set black"),
                },
            };

            match is_pressed {
                ButtonStatus::Pressed => button.pressed(),
                ButtonStatus::Depressed => button.depressed(),
                _ => {}
            }
        }
    }
}

pub struct Profile {
    brightness: u8,
    buttons: Vec<Button>,
}

///This is the stuct which is loaded into profiles to make interacting with behavior easier
pub struct Button {
    text: Option<String>,
    rgb: Option<[u8; 3]>,
    image: Option<DynamicImage>,
    // font: Option<rusttype::Font>,
    behavior: Option<Box<dyn Protocol>>,
}

impl Button {
    pub fn new(
        text: Option<String>,
        rgb: Option<[u8; 3]>,
        image: Option<DynamicImage>,
        behavior: Option<Box<dyn Protocol>>,
    ) -> Self {
        Self {
            text: text,
            rgb: rgb,
            image: image,
            // font: Some(rusttype::Font::try_from_bytes(include_bytes!()).unwrap()),
            behavior: behavior,
        }
    }

    fn empty() -> Self {
        Button {
            text: None,
            rgb: None,
            image: None,
            // font: None,
            behavior: None,
        }
    }

    fn update(&mut self) -> Option<Instruction> {
        let proto = self.behavior.as_ref().unwrap_or(return None);

        self.text = Some(proto.get_text()).unwrap_or(self.text);
        self.rgb = Some(proto.get_rgb()).unwrap_or(self.rgb);
        self.image = Some(proto.get_image()).unwrap_or(self.image);
        
        proto.get_instruction_request()
    }

    fn pressed(&mut self) {
        match self.behavior.as_mut() {
            Some(b) => b.pressed(),
            None => return,
        }
    }

    fn depressed(&mut self) {
        match self.behavior.as_mut() {
            Some(b) => b.depressed(),
            None => return,
        }
    }
}

///This is for instructions to the streamdeck that the plugins cant handle such as profile changes
enum Instruction {
    ChangeProfile(String),
}

pub trait Protocol: Debug {
    fn pressed(&mut self);
    fn depressed(&mut self) {
        return;
    }
    fn get_image(&self) -> Option<DynamicImage> {
        None
    }
    fn get_text(&self) -> Option<String> {
        None
    }
    fn get_rgb(&self) -> Option<[u8; 3]> {
        None
    }
    //TODO: for goodness sakes figure out how to implment this
    // fn get_font();
    fn get_instruction_request(&self) -> Option<Instruction> {
        None
    }
}
