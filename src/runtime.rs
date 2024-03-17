use std::{thread, sync::{Arc, Mutex}};

use clap::ArgMatches;

use crate::{networking::Network, structs::data::{Node, GlobalState}};

pub fn setup(tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>, m: &ArgMatches) {
    if let Some(port) = m.get_one::<String>("port") {
        let default_ip = "127.0.0.1".to_string();
        let ip = m.get_one::<String>("ip").unwrap_or(&default_ip);
        let mut net = Network::new(ip.to_string(), port.to_string());
        if let Some(network) = m.get_one::<String>("network") {
            let (ip, port) = network.split_once(":").unwrap();
            net.connect(ip.to_string(), port.to_string());
        };
        net.add_handler("display".to_string(), |message, _, _| {
            println!("{}", message.content);
        });
        let tree = Arc::clone(&tree);
        let global = Arc::clone(&global);
        thread::spawn(move || {
            net.listen(tree, global);
        });
    };
    loop {}
}

