mod parsing;
mod http;

mod structs;

use structs::data::{GlobalState, Node};
use std::{thread, sync::{Mutex, Arc}, io::{stdin, prelude::*, BufReader}, net::TcpListener};

use parsing::{parse_config, parse_raw_config};
use http::run_async_server;

fn parse_into_arc_mutex(file_name: &str) -> (Arc<Mutex<Node>>, Arc<Mutex<GlobalState>>) {
    match parse_config(file_name) {
        Ok((tree, global)) => (
            Arc::new(Mutex::new(tree)),
            Arc::new(Mutex::new(global))
            ),
        Err(err) => panic!("{}", err)
    }
}

fn listen_for_std(file_name: &str, tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>) {
    let stdobj = stdin();
    for line in stdobj.lock().lines() {
        let line = line.unwrap();
        if line.starts_with("restart") {
            match parse_config(file_name) {
                Ok((t, g)) => {
                    *tree.lock().unwrap() = t;
                    *global.lock().unwrap() = g;
                },
                Err(err) => panic!("{}", err)
            };
            println!("Restarted ✔");
        }
    }
}

fn apply_tcp_command(from: String, method: String, data: String, tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>) {
    match method.as_str() {
        "update" => {
            match parse_raw_config(data) {
                Ok((t, g)) => {
                    *tree.lock().unwrap() = t;
                    *global.lock().unwrap() = g;
                },
                Err(err) => panic!("{}", err)
            };
            println!("Updated configs from {}", from)
        },
        &_ => {}
    }
}

fn listen_for_tcp(port: u16, tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let mut buff = BufReader::new(&stream);
        // structure:
        // FROM(string) METHOD(verb, also string) DATA_LENGTH(uszie)
        // DATA
        let mut data = String::new();
        buff.read_line(&mut data).unwrap();
        let mut data = data.split_whitespace();
        let from = data.next().unwrap();
        let method = data.next().unwrap();
        let data_length: u64 = data.next().unwrap().parse().unwrap();
        let mut body = String::new();
        buff.take(data_length).read_to_string(&mut body).unwrap();
        let tree = Arc::clone(&tree);
        let global = Arc::clone(&global);
        apply_tcp_command(from.to_string(), method.to_string(), body, tree, global);

    }
}

fn main() {
    let version = "0.0.1".to_string();
    println!("DevSync v{}", version);

    let args: Vec<String> = std::env::args().collect();

    let file_name = args.get(1).unwrap_or(&"setup.json".to_string()).to_string();
    
    let (tree, global) = parse_into_arc_mutex(&file_name);
    {
        let tree = Arc::clone(&tree);
        let global = Arc::clone(&global);
        thread::spawn(move || {
            run_async_server(global, tree);
        });
    }
    {
        let tree = Arc::clone(&tree);
        let global = Arc::clone(&global);
        thread::spawn(move || {
            listen_for_tcp(1702, tree, global)
        });
    }
    listen_for_std(&file_name, tree, global);
    
}
