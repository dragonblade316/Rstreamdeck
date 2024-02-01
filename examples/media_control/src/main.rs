use mpris::Player;
use RstreamdeckPluginLib::{ButtonDesc, PluginAPI, ButtonEvent};
use std::{collections::HashMap, rc::Rc, cell::RefCell};



struct MController {
    finder: mpris::PlayerFinder,
    player: Option<mpris::Player>,
}

impl MController {
    fn new() -> Result<Self, ()> {
        let finder = mpris::PlayerFinder::new().unwrap();
        let player = finder.find_active().ok();

        Ok(MController {
            finder,
            player,
        })
    }

    fn update(&mut self) {
        self.player = self.finder.find_active().ok();
    }
    
    fn isplaying(&mut self) -> bool {
        self.update();
        match &self.player {
            Some(i) => i.get_playback_status().ok().eq(&Some(mpris::PlaybackStatus::Playing)),
            _ => return false
        }
    }

    fn toggle_player(&mut self) {
        self.update();
        match &self.player {
            Some(i) => i.play_pause(),
            None => return
        };
    }

    fn skip(&mut self) {
        self.update();
        match &self.player {
            Some(i) => i.next(),
            None => return
        };
    }

    fn back(&mut self) {
        self.update();
        match &self.player {
            Some(i) => i.previous(),
            None => return
        };
    }
}

//mute and volume can wait
enum Button {
    Volume(i32),
    Skip,
    Back,
    PlayPause,
    Mute,
    MuteMic
}

const PLAYPAUSEFONTSIZE: f32 = 84.0;
const PLAYPAUSETEXTOFFSET: (i32, i32) = (0, 15);

const SKIPBACKFONTSIZE: f32 = 64.0;
const SKIPBACKTEXTOFFSET: (i32, i32) = (0, 0);

fn main() {
    let descs: Vec<ButtonDesc> = vec![
    ];
     
    
    let mut buttons: Rc<RefCell<HashMap<u8, Button>>> = Rc::new(RefCell::new(HashMap::new()));
    let contrller = Rc::new(RefCell::new(MController::new().unwrap()));

    let mut api = PluginAPI::new(
        "media_control",
        Some(""),
        Some("dragonblade316"),
        descs,
        HashMap::new(),
        |ctx, id, button, position, opts| {
            ctx.send_rgb(id, [50, 0, 0]);
            match button.unwrap_or("play-pause".to_string()).as_str() {
                "play-pause" => {
                    ctx.send_text(id, "󰏤", Some(PLAYPAUSEFONTSIZE), Some(PLAYPAUSETEXTOFFSET));
                    println!("creating play-pause");
                    
                    buttons.borrow_mut().insert(id, Button::PlayPause);
                },
                "skip" => {
                    ctx.send_text(id, "", Some(SKIPBACKFONTSIZE), Some(SKIPBACKTEXTOFFSET));
                    buttons.borrow_mut().insert(id, Button::Skip);
                },
                "back" => {
                    ctx.send_text(id, "", Some(SKIPBACKFONTSIZE), Some(SKIPBACKTEXTOFFSET));
                    buttons.borrow_mut().insert(id, Button::Back);
                },
                _ => {
                    buttons.borrow_mut().insert(id, Button::PlayPause);
                    ctx.send_rgb(id, [255,255,255]);
                    println!("yeah there was no button input so you probably did something dumb lol");
                },
            };
        },
        |ctx, event, id| {
            // ctx.send_rgb(id, [255,255,255]);
            println!("Event: {event:?} triggered");

            match event {
                ButtonEvent::Pressed => ctx.send_rgb(id, [0,0,0]),
                ButtonEvent::Depressed => {
                    match buttons.borrow_mut().get(&id) {
                        Some(i) => match i {
                            Button::PlayPause => {
                                contrller.borrow_mut().toggle_player()
                            },
                            Button::Skip => {
                                contrller.borrow_mut().skip()
                            },
                            Button::Back => {
                                contrller.borrow_mut().back()
                            },
                            _ => {},
                        }
                        _ => {},
                    }
                }
            }

        }
    ).unwrap();

    loop {
        api.update(); 
        let mut c = contrller.borrow_mut();
        c.update();

        for i in buttons.borrow().iter() {
            let (id, button) = &i;
            match button {
                Button::PlayPause => match c.isplaying() {
                    //lol
                    true => api.get_ctx().send_text(id.clone().clone(), "󰐊", Some(84.0), None),
                    false => api.get_ctx().send_text(id.clone().clone(), "󰏤", Some(84.0), None),
                },
                _ => {}
            }
        }
    }
}

// 󰐊 󰏤   󰝝 󰝞 󰝟 
