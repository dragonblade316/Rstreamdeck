use std::collections::HashMap;
use std::eprintln;
use std::fmt::Debug;
use std::println;
use std::time::Duration;
use image::ImageBuffer;
use image::Pixel;
use image::Rgb;
use image::RgbImage;
use image::Rgba;
use imageproc::drawing::text_size;
use imageproc::rgb_image;
use rusttype::Font::*;
use rusttype::Scale;
use streamdeck::Kind;

use crate::default_buttons::new_button_protocol;
use crate::plugin;
use anyhow::Result;
use image::DynamicImage;
use streamdeck::Colour;
use streamdeck::StreamDeck;
use streamdeck::TextOptions;
use crate::config::{StreamdeckProfileToml, StreamdeckConfig};
use crate::plugin::PluginManager;
use std::fs;
use std::io::Read;
use Rstreamdeck_lib::Instruction;

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
        let mut man = PluginManager::new(config.plugin_path).expect("failed to start plugin manager");

        let vendorid = 0x0FD9;
        let mut pid: Option<u16> = None;

        let hid = hidapi::HidApi::new().expect("could not open hidapi");

        hid.device_list().for_each(|i| {
            if i.vendor_id() == vendorid {
                pid = Some(i.product_id());
            }
        });

        let deck: StreamDeck;pub struct ImageBuf { /* private fields */ }
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

            fn load_default_font() -> rusttype::Font<'static> {
                info!("using default font");
                rusttype::Font::try_from_bytes(include_bytes!("../../assets/SpaceMonoNerdFont-Regular.ttf")).unwrap()
            }

            let font = match profile.font {
               Some(p) => {
                   if p.exists() {
                       match fs::File::open(p) {
                            Ok(mut f) => {
                                let mut buf: Vec<u8> = Vec::new();
                                f.read_to_end(&mut buf).expect("thing");
                                rusttype::Font::try_from_vec(buf).unwrap_or(load_default_font())
                            },
                            _ => load_default_font()
                       }
                   } else {
                       load_default_font()
                   }
               },
               None => load_default_font(),

            };

                
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
                        info!("No button configured at {index}");
                        continue;
                    }
                };

                //TODO: we need to test this
                let position: [u8; 2] = match deck.kind() {
                    Kind::Mini => [index % 3, index / 3],
                    _ => [index % 5, index / 5],
                };

                let plugin: Option<&plugin::Plugin>;

                let button = match bconfig.plugin {
                    Some(p) =>{ 
                        Some(man.get_button(index, p, bconfig.button, position, bconfig.opts).unwrap())
                    },
                    None => match bconfig.button {
                        Some(b) => new_button_protocol(b, bconfig.opts),
                        None => None,
                    },
                };

                let image = match bconfig.icon {
                    Some(t) => Rstreamdeck_lib::load_icon(t),
                    None => None,
                };

                buttons.push(Button::new(bconfig.text, bconfig.fontsize, bconfig.text_xoffset, bconfig.text_yoffset, bconfig.rgb, image, button));
            }

            profiles.insert(/*I can not beleve I am setting I am using to_string() on &string*/ name.to_string(), Profile {
                brightness: profile.brightness.unwrap_or(100),
                font,
                fontsize: profile.fontsize.unwrap_or(24.0),
                buttons,
            });
        }

        for i in profiles.keys() {
            println!("listing profile {}", i);
        }

        
        man.lock();
        info!("plugins locked");

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
            info!("switching to profile {profile}");
            self.current_profile = profile;
        }
    }

    ///a helper function for getting the current profile

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

            // println!("updating the plugin manager");
            self.manager.update();
            
            

            match is_pressed {
                ButtonStatus::Pressed => button.pressed(),
                ButtonStatus::Depressed => button.depressed(),
                _ => {}
            }


            // yes I see I'm using a match pateren for only one thing. Deal with it. 
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

            // let im = &image::DynamicImage::new_rgb8(72, 72);
            // let tx = ctx.text();
            //

            //will not render the button if no changes have been made to renderable data
            if button.changed == false {
                continue; 
            }
            button.changed = false;
            
            //<renderer>
            let mut im: DynamicImage;
            if let Some(i) = &button.image {
                im = i.clone();
            } else if let Some(i) = &button.rgb {
                im =  DynamicImage::from(ImageBuffer::from_pixel(72, 72, Rgb([i[0], i[1], i[2]])));
                
            } else {
                im = DynamicImage::new_rgba8(72, 72);
            }
            
            
            if let Some(t) = &button.text {
                let scale = Scale::uniform(button.fontsize.unwrap_or(profile.fontsize));
                let (text_w, text_h) = text_size(scale, &profile.font, t);


                im = DynamicImage::from(imageproc::drawing::draw_text(
                    &im, 
                    Rgba([255,255,255,255]), 
                    ((im.width() / 2) as i32 - (text_w / 2)) - button.text_xoffset.unwrap_or(0), 
                    ((im.height() /2)  as i32 - (text_h / 2)) - button.text_yoffset.unwrap_or(0), 
                    scale,
                    &profile.font, 
                    t
                ));
            }

            let e = self.deck.set_button_image(index, im);
            //</renderer>
        }
    }

    fn get_text_values(s: String) {

    }
}

pub struct Profile {
    brightness: u8,
    font: rusttype::Font<'static>,
    fontsize: f32,
    buttons: Vec<Button>,
}

///This is the stuct which is loaded into profiles to make interacting with behavior easier
pub struct Button {
    text: Option<String>,
    fontsize: Option<f32>,
    text_xoffset: Option<i32>,
    text_yoffset: Option<i32>,
    rgb: Option<[u8; 3]>,
    image: Option<DynamicImage>,
    behavior: Option<Box<dyn Protocol>>,

    changed: bool,
}

impl Button {
    pub fn new(
        text: Option<String>,
        fontsize: Option<f32>,
        text_xoffset: Option<i32>,
        text_yoffset: Option<i32>,
        rgb: Option<[u8; 3]>,
        image: Option<DynamicImage>,
        behavior: Option<Box<dyn Protocol>>,
    ) -> Self {
        Self {
            text: text,
            fontsize,
            text_xoffset,
            text_yoffset,
            rgb: rgb,
            image: image,
            behavior: behavior,
            changed: true,
        }
    }

    fn empty() -> Self {
        Button {
            text: None,
            fontsize: None,
            text_xoffset: None,
            text_yoffset: None,
            rgb: None,
            image: None,
            behavior: None,
            changed: false,
        }
    }

    fn update(&mut self) -> Option<Instruction> {
        // println!("button being updated");
        
        //I hate that this works
        let proto = match &mut self.behavior {
            Some(i) => i,
            None => return None,
        };


        //I know this looks bad but I think my prior implmentation could have been doing weird
        //stuff with memory so we are going to try this

        match proto.get_text() {
            Some(t) => {
                if self.text != Some(t.clone()) {
                    self.text = Some(t);
                    self.changed = true;
                }
            }
            _ => {}
        }

        match proto.get_fontsize() {
            Some(f) => {
                if self.fontsize != Some(f) {
                    self.fontsize = Some(f);
                    self.changed = true;
                }
            },
            _ => {}
        }

        match proto.get_text_offset() {
            Some(o) => {
                

                let (x, y) = o;

                if !(Some(x) == self.text_xoffset && Some(y) == self.text_yoffset) {
                    self.text_xoffset = Some(x);
                    self.text_yoffset = Some(y);
                    self.changed = true;
                }
            }
            _ => {}
        }

        match proto.get_rgb() {
            Some(rgb) => {
                if self.rgb == Some(rgb) {
                    self.rgb = Some(rgb);
                    self.changed = true;
                }
            },
            _ => {}
        }

        match proto.get_image() {
            Some(i) => {
                if self.image == Some(i.clone()) {
                    self.image = Some(i);
                    self.changed = true;
                }
            },
            _ => {}        
        }
        
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
    fn get_fontsize(&self) -> Option<f32> {
        None
    }
    fn get_text_offset(&self) -> Option<(i32, i32)> {
        None
    }
    fn get_rgb(&self) -> Option<[u8; 3]> {
        None
    }
    fn get_instruction_request(&mut self) -> Option<Instruction> {
        None
    }
}
