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

fn main() {
    let descs: Vec<ButtonDesc> = vec![
    ];
     
    
    let mut buttons: Rc<RefCell<HashMap<u8, Button>>> = Rc::new(RefCell::new(HashMap::new()));
    let contrller = Rc::new(RefCell::new(MController::new().unwrap()));

    let mut api = PluginAPI::new(
        "media_control",
        None,
        Some("dragonblade316"),
        descs,
        HashMap::new(),
        |ctx, id, button, position, opts| {
            ctx.send_rgb(id, [50, 0, 0]);
            match button.unwrap_or("play-pause".to_string()).as_str() {
                "play-pause" => {
                    ctx.send_text(id, "󰏤");
                    ctx.send_rgb(id, [255,100,100]);
                    println!("creating play-pause");
                    
                    buttons.borrow_mut().insert(id, Button::PlayPause);
                },
                "skip" => {
                    ctx.send_text(id, "");
                    buttons.borrow_mut().insert(id, Button::Skip);
                },
                "back" => {
                    ctx.send_text(id, "" );
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
            ctx.send_rgb(id, [255,0,0]);

            match event {
                ButtonEvent::Pressed => ctx.send_rgb(id, [0,255,0]),
                ButtonEvent::Depressed => {
                    ctx.send_rgb(id, [0,0,255]);
                    match buttons.borrow_mut().get(&id) {
                        Some(i) => match i {
                            Button::PlayPause => {
                                contrller.borrow_mut().toggle_player()
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
                    true => api.get_ctx().send_text(id.clone().clone(), "󰏤"),
                    false => api.get_ctx().send_text(id.clone().clone(), "󰐊"),
                },
                _ => {}
            }
        }
    }
}

// 󰐊 󰏤   󰝝 󰝞 󰝟 
