mod config;
mod default_buttons;
mod hardware;
mod plugin;

#[macro_use] extern crate log;

use std::{
    clone,
    os::unix::thread,
    println,
    process::Command,
    thread::{spawn, Thread},
};

use clap::Parser;
use config::load_deck_from_config;
use simplelog::{CombinedLogger, TermLogger, Config, TerminalMode};
use streamdeck::{Colour, ImageOptions, StreamDeck, TextOptions, TextPosition};

extern crate streamdeck;

//just spesify the config file (honestly using clap may be a little overkill for this but whatever)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<String>,
}

fn main() {
    CombinedLogger::init(
        vec! [
            TermLogger::new(simplelog::LevelFilter::Warn, Config::default(), TerminalMode::Mixed, simplelog::ColorChoice::Auto),
            // WriteLogger::new(LevelFilter::Info, Config::default(), File::create("my_rust_binary.log").unwrap()),
        ]
    ).expect("logger could not be initalized");
    info!("Logger initialized");
    
    let args = Args::parse();

    let mut deck = load_deck_from_config(args.config).unwrap();
    info!("deck started");

    //this might al be moved to deck
    loop {
        //println!("update");
        deck.update()
    }
}
