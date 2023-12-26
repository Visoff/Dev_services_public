mod parsing;
mod http;

mod structs;

use std::{thread, sync::{Mutex, Arc}, io::{stdin, prelude::*}};

use parsing::parse_config;
use http::run_async_server;

fn main() {
    let file_name = "setup.json".to_string();
    let (tree, global) = match parse_config(file_name.clone()) {
        Ok((tree, global)) => (
            Arc::new(Mutex::new(tree)),
            Arc::new(Mutex::new(global))
            ),
        Err(err) => panic!("{}", err)
    };
    {
        let tree = Arc::clone(&tree);
        let global = Arc::clone(&global);
        thread::spawn(move || {
            run_async_server(global, tree);
        });
    }
    loop {
        let stdobj = stdin();
        for line in stdobj.lock().lines() {
            let line = line.unwrap();
            if line.starts_with("restart") {
                match parse_config(file_name.clone()) {
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
}
