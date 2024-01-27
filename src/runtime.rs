use std::{sync::{Arc, Mutex}, net::TcpListener, io::{BufReader, BufRead, Read, stdin}};

use clap::ArgMatches;

use crate::{networking::Network, structs::data::{Node, GlobalState}, parsing::{parse_raw_config, parse_config}};


fn listen_for_std(tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>) {
    let filename = "setup.json";
    let stdobj = stdin();
    for line in stdobj.lock().lines() {
        let line = line.unwrap();
        if line.starts_with("restart") {
            match parse_config(filename) {
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

pub fn setup(tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>, m: &ArgMatches) {
    let default_port = "1702".to_string();
    let port = m.get_one::<String>("port").unwrap_or(&default_port);
    let mut net = Network::new("127.0.0.1".to_string(), port.to_string());
    if let Some(network) = m.get_one::<String>("network") {
        let (ip, port) = network.split_once(":").unwrap();
        net.connect(ip.to_string(), port.to_string());
    }
    net.listen();
}
