use mpris::Player;
use RstreamdeckPluginLib::{ButtonDesc, PluginAPI};
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
            
            match button.unwrap_or("play-pause".to_string()).as_str() {
                "play-pause" => {
                    ctx.send_text(id, "󰏤");
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
                    println!("yeah there was no button input so you probably did something dumb lol");
                },
            };
        },
        |ctx, event, id| {
            match event {
                RstreamdeckPluginLib::ButtonEvent::Pressed => {
                    let button = buttons.borrow().get(&id).unwrap_or(return);
                    
                    match button {
                        &Button::PlayPause => contrller.borrow_mut().toggle_player(),
                        &Button::Skip => contrller.borrow_mut().skip(),
                        &Button::Back => contrller.borrow_mut().back(),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    ).unwrap();

    loop {
        //api.update(); 
        //contrller.borrow_mut().update();
    }
}

// 󰐊 󰏤   󰝝 󰝞 󰝟 
