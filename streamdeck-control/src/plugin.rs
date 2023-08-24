use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use Rstreamdeck_lib::ClientToServerMessage;
//use
///An interface for communicating with plugins
pub struct Plugin {
    socket: UnixStream,
    name: String,
    desc: Option<String>,
    buttons: Vec<String>,
}

impl Plugin {
    pub fn new(connection: UnixStream, desc: Rstreamdeck_lib::InitialReport) -> Self {
        Plugin {
            socket: connection,
            name: desc.name,
            desc: desc.desc,
            buttons: desc.buttons,
        }
    }

    pub fn spawn_button(pos: [u8; 2], button: String) {
        let id = pos[0] * pos[1];
    }

    fn send_message() {}

    fn process_message(message: ClientToServerMessage) {
        match message {
            ClientToServerMessage::ASSETREPORT(assets) => {}
            ClientToServerMessage::INITIALREPORT(report) => {}
            ClientToServerMessage::ERROR => {}
        }
    }

    fn update() {}
}

enum plugin_error {
    BAD_INITIAL_REPORT,
}

//will implment the protocol that allows plugins to be buttons
struct PluginButton {
    id: u8,
    text: Option<String>,
    rgb: Option<[u8; 3]>,
    image: Option<image::DynamicImage>,
}

///a system to register and manage plugins
pub struct PluginManager {
    listener: UnixListener,
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let socket_path = xdg::BaseDirectories::new()?
            .place_runtime_file("plugin_socket")
            .expect("could not place file");

        let socket_path = PathBuf::from("/tmp/rdeck.sock");
        let listener = UnixListener::bind(socket_path).expect("socket not bound");

        Ok(Self {
            listener: listener,
            plugins: HashMap::new(),
        })
    }

    pub fn load_plugin(&mut self, plugin: String) -> Result<&Plugin> {
        let path = xdg::BaseDirectories::new()
            .unwrap()
            .find_config_file(format!("/plugin/{}", plugin));

        let program = Command::new(path.unwrap()).spawn();

        let (mut plugin_socket, _) = self.listener.accept().unwrap();

        let mut buf: String = String::default();
        plugin_socket.read_to_string(&mut buf);

        let plugin_descriptor: Rstreamdeck_lib::ClientToServerMessage =
            serde_json::from_str(buf.as_str()).unwrap();

        match plugin_descriptor {
            Rstreamdeck_lib::ClientToServerMessage::INITIALREPORT(i) => {
                let plugin = Plugin::new(plugin_socket, i.clone());
                self.plugins.insert(i.name.clone(), plugin);

                //the unwrap should be fine here
                return Ok(&self.plugins.get(&i.name).unwrap());
            }
            _ => {
                /*plugin_socket.write_all(
                    serde_json::to_string(Rstreamdeck_lib::ClientToServerMessage::ERROR)?
                        .as_bytes(),
                );*/
                return Err(anyhow!("bad initial report"));
            }
        }
    }

    pub fn show_avalible_plugins() {
        let plugin_paths = xdg::BaseDirectories::new()
            .expect("wth")
            .find_config_files("plugins");

        unimplemented!()
    }

    fn accept_connections(&mut self) {
        let connections = self.listener.incoming().into_iter();

        for i in connections {
            match i {
                Ok(t) => {}
                Err(e) => {}
            }
        }
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        std::fs::remove_file("tmp/rdeck.sock");
    }
}
