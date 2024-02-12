use std::{thread, sync::{Arc, Mutex}, io::{BufRead, stdin}};

use clap::ArgMatches;

use crate::{networking::Network, structs::data::{Node, GlobalState}, parsing::parse_config};

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

pub fn setup(tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>, m: &ArgMatches) {
    if let Some(port) = m.get_one::<String>("port") {
        let mut net = Network::new("127.0.0.1".to_string(), port.to_string());
        if let Some(network) = m.get_one::<String>("network") {
            let (ip, port) = network.split_once(":").unwrap();
            net.connect(ip.to_string(), port.to_string());
        };
        thread::spawn(move || {
            net.listen();
        });
    };
    listen_for_std(Arc::clone(&tree), Arc::clone(&global));
}

