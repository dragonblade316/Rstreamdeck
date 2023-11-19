use std::{collections::HashMap, fmt::format, format};

use RstreamdeckPluginLib::PluginAPI;



struct Button {

}

fn main() {
    let buttons: HashMap<u8, Button> = HashMap::new();

    let descs = Vec<>;

    PluginAPI::new(
        format!("example"),
        Some(format!("An example plugin for Rstreamdeck")),
        Some(format!("dragonblade316")),
        button_desc,
        |api, id, button, position, opts| {

        },
        |api, id| {

        },
        |api, id| {

        },

    );



    
}
