use crate::hardware::{self, Protocol};
use anyhow::{anyhow, Result};
use base64::Engine;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{eprint, todo};
use Rstreamdeck_lib::{ClientToServerMessage, NewButton, ServerToClientMessage, ButtonDesc};
use ctrlc;

//use
///An interface for communicating with plugins
#[derive(Debug)]
pub struct Plugin {
    socket: UnixStream,
    name: String,
    desc: Option<String>,
    buttons: Vec<ButtonDesc>,
    active_buttons: HashMap<u8, PluginButtonInterface>,
    button_press_tx: Sender<press_state>,
    button_press_rx: Receiver<press_state>,
    used: bool,
}
//TODO
impl Plugin {
    pub fn new(connection: UnixStream, desc: Rstreamdeck_lib::InitialReport) -> Self {
        let (tx, rx) = channel();

        Plugin {
            socket: connection,
            name: desc.name,
            desc: desc.desc,
            buttons: desc.buttons,
            active_buttons: HashMap::new(),
            button_press_tx: tx,
            button_press_rx: rx,
            used: false,
        }
    }

    pub fn spawn_button(
        &mut self,
        id: u8,
        button: Option<String>,
        position: [u8; 2],
        opts: Option<HashMap<String, String>>,
    ) -> Result<Box<dyn Protocol>> {

        let data = ServerToClientMessage::NEWBUTTON(NewButton {
            id,
            button,
            position,
            opts,
        });

        println!("new button report sending");
        Rstreamdeck_lib::write_string_to_rdeck_socket(
            &mut self.socket,
            serde_json::to_string(&data).expect("json error"),
        );

        let text: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let fontsize: Arc<Mutex<Option<f32>>> = Arc::new(Mutex::new(None));
        let text_offset: Arc<Mutex<Option<(i32, i32)>>> = Arc::new(Mutex::new(None));
        let rgb: Arc<Mutex<Option<[u8; 3]>>> = Arc::new(Mutex::new(None));
        let image: Arc<Mutex<Option<image::DynamicImage>>> = Arc::new(Mutex::new(None));

        self.active_buttons.insert(
            id,
            PluginButtonInterface {
                id,
                image: image.clone(),
                text: text.clone(),
                fontsize: fontsize.clone(),
                text_offset: text_offset.clone(),
                rgb: rgb.clone(),
            },
        );

        Ok(Box::new(PluginButton {
            id,
            image: image.clone(),
            text: text.clone(),
            fontsize: fontsize.clone(),
            text_offset: text_offset.clone(),
            rgb: rgb.clone(),
            plugin: self.button_press_tx.clone(),
        }))
    }

    //TODO: make this multithreaded
    fn update(&mut self) {
        //read from socket
        self.socket.set_read_timeout(Some(Duration::new(0, 5000)));
        
        let json_str = Rstreamdeck_lib::read_string_from_rdeck_socket(&mut self.socket).unwrap();
        // println!("recived json");

        self.check_button_presses();

        if json_str == format!("") {
            return;
        }

        let json = serde_json::from_str::<ClientToServerMessage>(&json_str.as_str()).unwrap();

        match json {
            ClientToServerMessage::BUTTONREPORT(b) => {
                let button = self.active_buttons.get_mut(&b.id).unwrap();

                let image = match b.image {
                    Some(im) => {
                        let b64 = base64::engine::general_purpose::STANDARD_NO_PAD;
                        Some(image::load_from_memory(&b64.decode(im).unwrap()).expect("somebody did not transcode the image properly"))
                    }
                    None => None,
                };

                let mut im = button.image.lock().unwrap();
                let mut rgb = button.rgb.lock().unwrap();
                let mut text = button.text.lock().unwrap();
                let mut fontsize = button.fontsize.lock().unwrap();
                let mut text_offset = button.text_offset.lock().unwrap();

                
                *im = image;
                *rgb = b.rgb;
                // *text = b.text;

                match b.text {
                    Some(t) => *text = Some(t),
                    None => {}
                }

                match b.fontsize {
                    Some(f) => *fontsize = Some(f),
                    None => {},
                }
                
                match b.text_offset {
                    Some(o) => *text_offset = Some(o),
                    None => {}
                }

            }
            ClientToServerMessage::ERROR(i) => {},
            _ => {}
            
        }
    }

    fn check_button_presses(&mut self) {
        
        let state = match self.button_press_rx.try_recv() {
            Ok(i) => i,
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Empty => return,
                std::sync::mpsc::TryRecvError::Disconnected => panic!("WTH is going on here! why is this on with no buttons reporting to it?"),
            },
        };

        match state {
            press_state::PRESSED(id) => {
                Rstreamdeck_lib::send_message_to_plugin(&mut self.socket, ServerToClientMessage::PRESSED(id));
            }
            press_state::DEPRESSED(id) => Rstreamdeck_lib::write_string_to_rdeck_socket(
                &mut self.socket,
                serde_json::to_string(&ServerToClientMessage::DEPRESSED(id)).unwrap(),
            ),
        }
    }
}

enum press_state {
    PRESSED(u8),
    DEPRESSED(u8),
}

enum plugin_error {
    BAD_INITIAL_REPORT,
}

//will implment the protocol that allows plugins to be buttons
//also made it thread safe for later. nice
#[derive(Debug)]
struct PluginButton {
    id: u8,
    text: Arc<Mutex<Option<String>>>,
    fontsize: Arc<Mutex<Option<f32>>>,
    text_offset: Arc<Mutex<Option<(i32, i32)>>>,
    rgb: Arc<Mutex<Option<[u8; 3]>>>,
    image: Arc<Mutex<Option<image::DynamicImage>>>,
    plugin: Sender<press_state>,
}

impl Protocol for PluginButton {
    fn pressed(&mut self) {
        println!("plugin button pressed");
        self.plugin.send(press_state::PRESSED(self.id)).unwrap();
    }
    fn depressed(&mut self) {
        self.plugin.send(press_state::DEPRESSED(self.id)).unwrap();
    }
    fn get_image(&self) -> Option<image::DynamicImage> {
        //perhaps set a hashing system to make this cheaper
        self.image.lock().unwrap().clone()
        //self.image.().unwrap().clone()
    }
    fn get_rgb(&self) -> Option<[u8; 3]> {
        let thing = self.rgb.lock().unwrap().clone();
        thing
    }
    fn get_text(&self) -> Option<String> {
        let thing = self.text.lock().unwrap().clone();
        thing
    }
    fn get_fontsize(&self) -> Option<f32> {
        self.fontsize.lock().unwrap().clone()
    }
    fn get_text_offset(&self) -> Option<(i32, i32)> {
        self.text_offset.lock().unwrap().clone()
    }
}

#[derive(Debug)]
struct PluginButtonInterface {
    id: u8,
    text: Arc<Mutex<Option<String>>>,
    fontsize: Arc<Mutex<Option<f32>>>,
    text_offset: Arc<Mutex<Option<(i32, i32)>>>,
    rgb: Arc<Mutex<Option<[u8; 3]>>>,
    image: Arc<Mutex<Option<image::DynamicImage>>>,
}

///a system to register and manage plugins
pub struct PluginManager {
    listener: UnixListener,
    plugins: HashMap<String, (Plugin, std::process::Child)>,
}

impl PluginManager {
    pub fn new(plugin_path: PathBuf) -> Result<Self> {

        let socket_path = PathBuf::from("/tmp/rdeck.sock");

        if socket_path.exists() {
            std::fs::remove_file("/tmp/rdeck.sock").expect("failed to destroy file");
        }


        let listener = UnixListener::bind(socket_path).expect("socket not bound");

        //loads all plugins. the unused ones will be unloaded by self.lock().
        //The reason for this is the plugin filenames dont seem reliable enough to base the
        //buttonloading on. the plugin initial reports are better
        //Todo: make a cache file that will be stored in the plugins folder that keeps track of
        //which filenames belong to which plugins
        let plugins: HashMap<String, (Plugin, std::process::Child)> = HashMap::new();

        let mut man = Self {
            listener: listener,
            plugins: HashMap::new(),
        };

                //makes the Drop impl irrelevent.
        ctrlc::set_handler(|| {
            std::fs::remove_file("/tmp/rdeck.sock").expect("failed to destroy file");
            //for some reason setting a handler makes the program not exit on SIGINT. 
            std::process::exit(0);
        }).expect("failed to load close handler");


        man.load_plugin(plugin_path);

        Ok(man)
    }

    //this function does not need to be here but it makes it easier
    fn load_plugin(&mut self, plugin_dir: PathBuf) {
        fn send_error(socket: &mut UnixStream) {
            Rstreamdeck_lib::write_string_to_rdeck_socket(
                socket,
                serde_json::to_string(&Rstreamdeck_lib::ServerToClientMessage::Error(Rstreamdeck_lib::ServerError::BAD_INITIAL_REPORT)).unwrap(),
            );
        }

        //rewrite this to support custom config dirs (or maybe not)
        let plugins = xdg::BaseDirectories::new()
            .unwrap()
            .find_config_files("plugins");



        if !plugin_dir.exists() || !plugin_dir.is_dir() {
            let _ = std::fs::create_dir(&plugin_dir);
        }



        println!("loading plugins");
        for i in std::fs::read_dir(plugin_dir).unwrap() {
            
            let plugin_path = i.unwrap().path();

            let debug_string = plugin_path.to_str().unwrap(); 
            println!("starting plugin {}", debug_string);
            let child = Command::new(plugin_path).spawn().unwrap();
            println!("started successfully");
            //     .unwrap_or({
            //     eprintln!("plugin somehow does not exist");
            //     continue;
            // });

            let (mut socket, addr) = self.listener.accept().unwrap();
            
            println!("awaiting initial report");
            let initial_report = serde_json::from_str::<ClientToServerMessage>(
                &Rstreamdeck_lib::read_string_from_rdeck_socket(&mut socket)
                    .unwrap()
                    .as_str(),
            )
            .unwrap();
            println!("recived initial report");

            let key: String;

            let plugin = match initial_report {
                Rstreamdeck_lib::ClientToServerMessage::INITIALREPORT(s) => {
                    key = s.name.clone();
                    Plugin::new(socket, s)
                }
                _ => {
                    send_error(&mut socket);
                    continue;
                }
            };

            println!("plugin loaded. name is {}", &plugin.name);

            self.plugins.insert(key, (plugin, child));
        }
    }

    ///an easy way to spawn plugin button protocols
    pub fn get_button(
        &mut self,
        id: u8,
        plugin: String,
        button: Option<String>,
        position: [u8; 2],
        opts: Option<HashMap<String, String>>,
    ) -> Result<Box<dyn hardware::Protocol>> {
        match self.plugins.get_mut(&plugin) {
            Some(bp) => {
                let (p, _) = bp;
                p.used = true;
                p.spawn_button(id, button, position, opts)
            }
            None => Err(anyhow!("plugin {} not found", plugin)),
        }
    }

    ///tells the manager that configuration is done and no new buttons will be created. this
    ///results in unused plugins being killed
    pub fn lock(&mut self) {
        let mut remove_list: Vec<String> = Vec::new();

        for i in self.plugins.iter() {
            let (id, (plugin, child)) = i;

            if plugin.used == false {
                println!("removing: {}", id);
                remove_list.push(id.clone());
            }
        }

        for i in remove_list {
            let (_, mut child) = self.plugins.get(&i).unwrap_or(continue);
            child.kill();
            self.plugins.remove(&i);
        }
    }

    //this is probably a temperary function until I implment multithreading
    pub fn update(&mut self) {
        for i in self.plugins.iter_mut() {
            let (name, (plugin, child)) = i;


            plugin.update();

        }
    }
}


