mod config;
mod plugin;

use std::{thread::{Thread, spawn}, os::unix::thread, clone, process::Command };

use clap::Parser;
use config::Deckstate;
use streamdeck::{StreamDeck, TextPosition, TextOptions, Colour, ImageOptions};

extern crate streamdeck;

//just spesify the config file (honestly using clap may be a little overkill for this but whatever)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<String>
}

fn main() {
    let args = Args::parse();

    let mut config = config::load_config(args.config).unwrap();
    
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
            deck = streamdeck::StreamDeck::connect_with_hid(&hid, vendorid, id, None).expect("could not connect to streamdeck");
        } 
        None => panic!("no streamdeck detected")
    }


    println!("{:?}", deck.image_size());

        

    load_button_config(&mut deck, &config);



    loop {
        let state = deck.read_buttons(None).unwrap();

        for i in state.iter().zip(&mut config.buttons) {
            let (button, config): (&u8, &dyn Rstreamdeck_lib::Button) = i;

            
        }
    }
}

//I cant believe this crap works
fn load_button_config(deck: &mut StreamDeck, config: &Deckstate ) {
    deck.set_brightness(config.brightness).unwrap_or(deck.set_brightness(100).unwrap_or(eprintln!("Setting brightness has gone very wrong. but I guess the app is still running")));

    println!("{:?}", deck.kind().keys());

    for i in 0..deck.kind().keys() {
        let conf = &config.buttons[usize::from(i)];


        println!("{:?}", i);


        match &conf.icon {
            Some(image) => { 
                println!("image size is {:?}, {:?}", image.width(), image.height());
                deck.set_button_image(i, image.clone())
            },
            None => match &conf.rgb {
                Some(rgb) => deck.set_button_rgb(i, &Colour { r: rgb[0], g: rgb[1], b: rgb[2] }),
                None => deck.set_button_rgb(i, &Colour { r: 0, g: 0, b: 0 }),
            }
        };
    }
}
