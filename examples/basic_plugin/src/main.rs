use std::{collections::HashMap, fmt::format, format};

use RstreamdeckPluginLib::PluginAPI;
use RstreamdeckPluginLib::ButtonDesc;



struct Button {

}

fn main() {
    //when a new button is created we need to store it. This lib makes you store it so you have
    //maxumum flexibility at the cost of complexity
    let buttons: HashMap<u8, Button> = HashMap::new();

    let descs: Vec<ButtonDesc> = Vec::new();

    PluginAPI::new(
        format!("example"),
        Some(format!("An example plugin for Rstreamdeck")),
        Some(format!("dragonblade316")),
        descs,
        HashMap::new(),
        |ctx, id, _button, _position, _opts| {
            ctx.send_rgb(id, [30, 0, 0])
        },

        |ctx, _event, id| {
            ctx.send_rgb(id, [0, 0, 30]) 
        },

    );
}
