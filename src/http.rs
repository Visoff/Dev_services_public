use crate::structs::{data::{Node, GlobalState}, http::{Request, Response}};

use std::{io::prelude::*, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread};

pub fn handle_request(mut stream:TcpStream, global: &GlobalState, tree: &Node) {
    if let Some(mut req) = Request::from_stream(&stream) {
        let (node, remained_uri) = tree.search(req.uri);
        req.uri = remained_uri;
        let res = match node.value.as_ref() {
            Some(component) => component.call(global, req),
            None => Response::not_found()
        };
        stream.write_all(res.into_string().as_bytes()).unwrap();
    }
    
}

pub fn run_async_server(global: Arc<Mutex<GlobalState>>, tree: Arc<Mutex<Node>>) {
    let pure_global = global.lock().unwrap();
    if let Some(exposed) = &pure_global.exposed {
        let server = TcpListener::bind(&exposed.stringify()).unwrap();
        drop(pure_global);
        for stream in server.incoming() {
            let global = Arc::clone(&global);
            let tree = Arc::clone(&tree);
            thread::spawn(move || {
                match stream {
                    Ok(stream) => {
                        handle_request(stream, &global.lock().unwrap(), &tree.lock().unwrap());
                    },
                    Err(_) => println!("Error")
                };
            });
        }
    }
}

pub fn run_server(global: &GlobalState, tree: &Node) {
    if let Some(exposed) = &global.exposed {
        let server = TcpListener::bind(&exposed.stringify()).unwrap();
        loop {
            match server.accept() {
                Ok((stream, _addr)) => handle_request(stream, global, tree),
                Err(_) => println!("Error")
            };
        }
    }
}

