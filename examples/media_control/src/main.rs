use mpris::Player;
use RstreamdeckPluginLib::{ButtonDesc, PluginAPI};
use std::collections::HashMap;



struct MController {
    finder: mpris::PlayerFinder,
    player: Option<mpris::Player>,
}

impl MController {
    fn new() -> Result<Self, ()> {
        let finder = mpris::PlayerFinder::new().unwrap_or(return Err);

        Ok(MController {
            finder,
            player: Some(finder.find_active().unwrap_or(return Err)),
        })
    }

    fn update(&mut self) {
        self.player = self.finder.find_active().ok();
    }
    
    fn isplaying(&mut self) -> bool {
        self.update();
        self.player.unwrap_or(return).get_playback_status().is_ok().eq(&mpris::PlaybackStatus::Playing)
    }

    fn toggle_player(&mut self) {
        self.update();
        self.player.unwrap_or(return).play_pause();
    }

    fn skip(&mut self) {
        self.update();
        self.player.unwrap_or(return).next();
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
    

    let mut buttons: HashMap<u8, Button> = HashMap::new();

    let api = PluginAPI::new(
        "media_control",
        None,
        Some("dragonblade316"),
        descs,
        HashMap::new(),
        |ctx, id, button, position, opts| {

            let button = button.unwrap_or("play-pause".to_string()).as_str();
            match button {
                "play-pause" => {
                    ctx.send_text(id, "󰏤");
                    buttons.insert(id, Button::PlayPause);
                },
                "skip" => {
                    ctx.send_text(id, "");
                    buttons.insert(id, Button::Skip);
                },
                "back" => {
                    ctx.send_text(id, "" );
                    buttons.insert(id, Button::Back);
                },
                _ => {},
            };
        },
        |ctx, event, id| {
            match event {
                RstreamdeckPluginLib::ButtonEvent::Pressed => {
                    let button = buttons.get(&id).unwrap_or(return);
                    
                    match button {
                        Button::PlayPause
                    }
                },
                _ => {}
            }
        }
    );

    loop {
                
    }
}

// 󰐊 󰏤   󰝝 󰝞 󰝟 
