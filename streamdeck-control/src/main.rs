mod config;
mod default_buttons;
mod hardware;
mod plugin;

use std::{
    clone,
    os::unix::thread,
    println,
    process::Command,
    thread::{spawn, Thread},
};

use clap::Parser;
use config::load_deck_from_config;
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
    let args = Args::parse();

    let mut deck = load_deck_from_config(args.config).unwrap();
    println!("deck works");

    //this might al be moved to deck
    loop {
        println!("update");
        deck.update()
    }
}
