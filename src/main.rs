mod parsing;
mod http;
mod structs;
mod runtime;

use structs::data::{GlobalState, Node};
use std::{thread, sync::{Mutex, Arc}};

use parsing::parse_config;
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
    runtime::setup(tree, global);
}
