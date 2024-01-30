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

    let mut api = PluginAPI::new(
        "example",
        Some("An example plugin for Rstreamdeck"),
        Some("dragonblade316"),
        descs,
        HashMap::new(),
        |ctx, id, _button, _position, _opts| {
            ctx.send_text(id, "hi", None, None);
            ctx.send_rgb(id, [30, 0, 0])
        },

        |ctx, _event, id| {
            ctx.send_text(id, "bye", None, None);
            ctx.send_rgb(id, [0, 0, 30]) 
        },
    ).unwrap();

    api.set_blocking(true);
    loop {   
        api.update();
    };
}
