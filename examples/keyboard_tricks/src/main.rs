use std::{collections::HashMap, borrow::Borrow};
use std::rc::Rc;
use std::cell::RefCell;

use RstreamdeckPluginLib::{PluginAPI};
use enigo::{Enigo, KeyboardControllable};

#[macro_use] extern crate log;

enum button {
    TEXT(String) 
}

fn main() {

    let mut buttons: Rc<RefCell<HashMap<u8, button>>> = Rc::new(RefCell::new(HashMap::new()));
    let mut hid = Enigo::new();
    
    
    let mut api = PluginAPI::new(
        "keyboard_tricks",
        None,
        Some("dragonblade316"), 
        vec![], 
        HashMap::new(), 
        |ctx, id, button, _position, opts| {
            match button {
                Some(b) => match b.as_str() {
                    "text" => {
                        let t = opts.unwrap_or({
                            error!("no opts");
                            return;
                        }).get(&"text".to_string()).unwrap_or({
                            error!("did not add text opt");
                            return
                        });

                        buttons.get_mut().insert(id, button::TEXT(t.to_owned()));
                    },

                    _ => {},
                },
                None => {},
            }
        
        }, 
        |ctx, event, id| {
            match event { 
                RstreamdeckPluginLib::ButtonEvent::Pressed => match buttons.borrow_mut().get(&id).unwrap_or(return) {
                    button::TEXT(t) => hid.key_sequence_parse(t.as_str()),
                    _ => {},
                }
                RstreamdeckPluginLib::ButtonEvent::Depressed => {},
            }
            
        }
    ).unwrap();

    loop {
        api.update(); 
    }
}
