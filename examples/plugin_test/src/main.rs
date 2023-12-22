use std::{os::unix::net::{UnixListener, UnixStream}, path::PathBuf, process::Command};
use Rstreamdeck_lib::{write_string_to_rdeck_socket, read_string_from_rdeck_socket, ServerToClientMessage, ClientToServerMessage};
use clap::Parser;
use std::io::{Read, Write};

#[derive(Parser)]
struct opts {
    plug_path: PathBuf
}

fn main() {
    let args = opts::parse();
   
    println!("started, testing plugin {}", args.plug_path.to_str().unwrap());
    let mut listener = UnixListener::bind("/tmp/rdeck.sock").unwrap();
    let mut child = Command::new(args.plug_path).spawn().unwrap();

    let (mut conn, _) = listener.accept().unwrap();
    
    let istr = read_string_from_rdeck_socket(&mut conn).unwrap();

    println!("{istr}");

    let ijson: Rstreamdeck_lib::ClientToServerMessage = serde_json::from_str(&istr).unwrap();

    match ijson {
        Rstreamdeck_lib::ClientToServerMessage::INITIALREPORT(i) => println!("{}", i.name),
        _ => println!("you did something dumb and should be sad lol"),
    };
    
    // let json = serde_json::to_string(&ServerToClientMessage::NEWBUTTON(Rstreamdeck_lib::NewButton {
    //     id: 0,
    //     button: None,
    //     opts: None,
    //     position: [0,0],
    // })).unwrap();
    //
    // write_string_to_rdeck_socket(&mut conn, json);
    //
    Rstreamdeck_lib::send_message_to_plugin(&mut conn, ServerToClientMessage::NEWBUTTON(Rstreamdeck_lib::NewButton {
        id: 0,
        button: None,
        opts: None,
        position: [0,0],
    }));

    let bstr = read_string_from_rdeck_socket(&mut conn).unwrap();
    let bjson: ClientToServerMessage = serde_json::from_str(bstr.as_str()).unwrap();

    match bjson {
        ClientToServerMessage::BUTTONREPORT(i) => {
            println!("{}", i.id);
            println!("{}", i.rgb.unwrap_or([0,0,0])[0 as usize]);
        }
        _ => println!("thing"),
    }

    child.kill();

    std::fs::remove_file("/tmp/rdeck.sock");
    
}
