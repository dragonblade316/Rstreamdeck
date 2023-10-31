use std::collections::HashMap;
use std::eprintln;
use std::fmt::Debug;
use std::println;
use std::time::Duration;

use crate::default_buttons::new_button_protocol;
use crate::plugin;
use anyhow::Result;
use image::DynamicImage;
use streamdeck::Colour;
use streamdeck::StreamDeck;

use crate::config::stdStreamdeckToml;
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
    pub fn new(config: stdStreamdeckToml) -> Result<Deck> {
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

        let button_vec = vec![
            config.b1, config.b2, config.b3, config.b4, config.b5, config.b6, config.b7, config.b8,
            config.b9, config.b10, config.b11, config.b12, config.b13, config.b14, config.b15,
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

        println!("deck creation successful");

        man.lock();

        //TODO: switch to profiles 
        Ok(Deck {
            deck,
            brightness: config.deck.brightness.unwrap_or(100 as u8),
            buttons: buttons,
            manager: man,
            last_state: Vec::new(),
        })
    }

    fn change_profile(&mut self, profile: String) {

    }

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

        for i in (0..self.deck.kind().keys())
            .zip(&mut self.buttons)
            .zip(status)
        {
            let ((index, button), is_pressed) = i;

            self.deck.set_brightness(self.brightness);

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
//#[derive(Debug)]
pub struct Button {
    text: Option<String>,
    rgb: Option<[u8; 3]>,
    image: Option<DynamicImage>,
    // font:
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
            behavior: behavior,
        }
    }

    fn empty() -> Self {
        Button {
            text: None,
            rgb: None,
            image: None,
            behavior: None,
        }
    }

    fn asset_call(&mut self) {
        let proto = self.behavior.as_ref().unwrap_or(return);

        self.text = Some(proto.get_text()).unwrap_or(self.text);
        self.rgb = Some(proto.get_rgb()).unwrap_or(self.rgb);
        self.image = Some(proto.get_image()).unwrap_or(self.image)
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
    // fn get_font();
}
