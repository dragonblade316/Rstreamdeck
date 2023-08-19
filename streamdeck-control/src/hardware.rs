use std::collections::HashMap;
use std::eprintln;

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
    brightness: u8,
    buttons: Vec<Button>,
    manager: PluginManager,
}

impl Deck {
    pub fn new(config: stdStreamdeckToml) -> Result<Deck> {
        let mut man = PluginManager::new()?;

        let vendorid = 0x0FD9;
        let mut pid: Option<u16> = None;

        let mut deck: StreamDeck;

        let hid = hidapi::HidApi::new().expect("could not open hidapi");

        hid.device_list().for_each(|i| {
            if i.vendor_id() == vendorid {
                pid = Some(i.product_id());
            }
        });

        match pid {
            Some(id) => {
                deck = streamdeck::StreamDeck::connect_with_hid(&hid, vendorid, id, None)
                    .expect("could not connect to streamdeck");
            }
            None => panic!("no streamdeck detected"),
        }

        let deck: StreamDeck;
        match pid {
            Some(id) => {
                deck = streamdeck::StreamDeck::connect_with_hid(&hid, vendorid, id, None)
                    .expect("could not connect to streamdeck");
            }
            None => panic!("no streamdeck detected"),
        }

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

            //TODO: I will need to improve the error handleing here in case something can be fixed
            //or done here
            match bconfig.plugin {
                Some(p) => plugin = man.load_plugin(p).ok(),
                None => plugin = None,
            }

            let image = match bconfig.icon {
                Some(t) => Rstreamdeck_lib::load_icon(t),
                None => None,
            };

            buttons.push(Button::new(
                bconfig.text,
                bconfig.rgb,
                image,
                bconfig.button,
                bconfig.opts,
                plugin,
            ));
        }

        Ok(Deck {
            deck,
            brightness: config.deck.brightness.unwrap_or(100 as u8),
            buttons: buttons,
            manager: man,
        })
    }

    pub fn update(&mut self) {
        for i in (0..self.deck.kind().keys())
            .zip(&mut self.buttons)
            .zip(self.deck.read_buttons(None).unwrap_or(vec![0; 64]))
        {
            let ((index, button), status) = i;

            self.deck.set_brightness(self.brightness);

            match &button.image {
                Some(im) => self.deck.set_button_image(index, im.clone()),
                None => match button.rgb {
                    Some(rgb) => self.deck.set_button_rgb(
                        index,
                        &Colour {
                            r: rgb[0],
                            g: rgb[1],
                            b: rgb[2],
                        },
                    ),
                    None => self
                        .deck
                        .set_button_rgb(index, &Colour { r: 0, g: 0, b: 0 }),
                },
            };

            button.pressed();
        }
    }
}

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
        button: Option<String>,
        opts: Option<HashMap<String, String>>,
        plugin: Option<&crate::plugin::Plugin>,
    ) -> Self {
        let behavior = match plugin {
            Some(p) => {
                unimplemented!()
            }
            None => {
                use crate::default_buttons::new_button_protocol;

                match button {
                    Some(i) => new_button_protocol(i, opts),
                    None => None,
                }
            }
        };

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
        let proto = self.behavior.unwrap_or(return);

        self.text = Some(proto.get_text()).unwrap_or(self.text);
        self.rgb = Some(proto.get_rgb()).unwrap_or(self.rgb);
        self.image = Some(proto.get_image()).unwrap_or(self.image)
    }

    fn pressed(&self) {
        self.behavior.unwrap_or(return).pressed();
    }
}

pub trait Protocol {
    fn pressed(&mut self);
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
