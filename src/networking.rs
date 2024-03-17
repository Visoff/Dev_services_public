use std::{time::Duration, thread::sleep, sync::{Arc, Mutex}, collections::HashMap, net::{TcpListener, TcpStream}, io::{BufReader, BufRead, Write, Read}};

use crate::structs::data::{Node, GlobalState};

#[derive(Clone)]
pub struct Message {
    pub from: String,
    pub method: String,
    pub params: HashMap<String, String>,
    pub content: String
}

impl Message {
    pub fn build(from: String, method: String, params: HashMap<String, String>, content: String) -> Self {
        Message {
            from,
            method,
            params,
            content,
        }
    }

    pub fn parse(stream: &TcpStream) -> Self {
        let mut reader = BufReader::new(stream);
        let mut buffer = String::new();

        reader.read_line(&mut buffer).unwrap();
        let mut data_line = buffer.split_whitespace();
        let from = data_line.next().unwrap();
        let method = data_line.next().unwrap();
        
        let mut params: HashMap<String, String> = HashMap::new();
        let mut content_length: usize = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            line = line.trim().to_string();
            if line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(": ").unwrap();
            if name == "Content-Length" {
                content_length = value.parse().unwrap();
                continue;
            }
            params.insert(name.to_string(), value.to_string());
        }
        let mut content = String::new();
        reader.take(content_length as u64).read_to_string(&mut content).unwrap();
        return Message {
            from: from.to_string(),
            method: method.to_string(),
            params,
            content
        };
    }

    pub fn as_string(&self) -> String {
        format!(
            "{} {}\r\n{}\r\n\r\n{}",
            self.from,
            self.method,
            {
                let mut new_map = self.params.clone();
                new_map.insert("Content-Length".to_string(), self.content.len().to_string());
                new_map.iter()
                    .map(|(k,  v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join("\r\n")
            },
            self.content
        )
    }

    pub fn send(&self, receiver: &NetworkNode) {
        let mut stream = TcpStream::connect(format!("{}:{}", receiver.ip, receiver.port)).unwrap();
        stream.write(self.as_string().as_bytes()).unwrap();
    }
}

pub struct Network {
    nodes: HashMap<String, NetworkNode>,
    my_id: String,
    handlers: HashMap<String, fn(Message, Arc<Mutex<Node>>, Arc<Mutex<GlobalState>>)>
}

#[derive(Debug)]
pub struct NetworkNode {
    ip: String,
    port: String,
}

impl Network {
    pub fn new(ip: String, port: String) -> Network {
        let mut network = Network {
            nodes: HashMap::new(),
            my_id: uuid::Uuid::new_v4().to_string(),
            handlers: HashMap::new()
        };
        network.nodes.insert(network.my_id.to_string(), NetworkNode { ip, port });
        network
    }

    pub fn get_my_node(&self) -> &NetworkNode {
        self.nodes.get(&self.my_id).unwrap()
    }

    pub fn connect(&mut self, ip: String, port: String) {
        let my_node = self.get_my_node();
        println!("Connecting to {}:{}...", ip, port);
        let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).unwrap();
        stream.write(
            format!("{} connect\r\nip: {}\r\nport: {}\r\n\r\n",
                    self.my_id,
                    my_node.ip,
                    my_node.port
                ).as_bytes()).unwrap();
        
    }

    pub fn add_handler(&mut self, method: String, handler: fn(Message, Arc<Mutex<Node>>, Arc<Mutex<GlobalState>>)) {
        self.handlers.insert(method, handler);
    }

    pub fn shout(&self, message: Message) {
        self.nodes.iter().for_each(|(_, node)| {
            message.send(node);
        });
    }

    pub fn request(&mut self, stream: TcpStream, tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>) {
        let message = Message::parse(&stream);
        match message.method.as_str() {
            "connect" => {
                println!("{}", stream.peer_addr().unwrap().ip());
                sleep(Duration::from_secs(1));
                let ip = message.params.get("ip").unwrap();
                let port = message.params.get("port").unwrap();
                let new_node = NetworkNode { ip: ip.to_string(), port: port.to_string() };
                Message::build(
                    self.my_id.to_string(),
                    "add_members".to_string(),
                    HashMap::new(),
                    self.nodes.iter()
                        .map(|(id, node)| format!("{} {}:{}", id, node.ip, node.port).to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                ).send(&new_node);
                let mut params: HashMap<String, String> = HashMap::new();
                params.insert("ip".to_string(), ip.to_string());
                params.insert("port".to_string(), port.to_string());
                params.insert("name".to_string(), message.from.to_string());
                let shout_message = Message::build(self.my_id.to_string(), "add_member".to_string(), params, "".to_string());
                self.shout(shout_message);
                self.nodes.insert(message.from.to_string(), new_node);
            },
            "add_member" => {
                let ip = message.params.get("ip").unwrap().to_string();
                let port = message.params.get("port").unwrap().to_string();
                let name = message.params.get("name").unwrap().to_string();
                self.nodes.insert(name, NetworkNode{ ip, port });
                println!("Added nodes\r\nnew once: {:?}", self.nodes);
            },
            "add_members" => {
                message.content.split("\n").for_each(|line| {
                    let (name, data) = line.split_once(" ").unwrap();
                    let (ip, port) = data.split_once(":").unwrap();
                    self.nodes.insert(name.to_string(), NetworkNode{ ip: ip.to_string(), port: port.to_string() });
                });
                println!("Added nodes\r\nnew once: {:?}", self.nodes);
            },
            "message" => {
                let content = message.content.to_string();
                self.shout(Message::build(self.my_id.to_string(), "display".to_string(), HashMap::new(), content.to_string()));
            },
            &_ => {
                if self.handlers.contains_key(&message.method) {
                    let handler = self.handlers.get(&message.method).unwrap();
                    handler(
                        message,
                        tree,
                        global
                    );
                } else {
                    println!("Unknown method: {}", message.method);
                }
            }
        }
    }

    pub fn listen(&mut self, tree: Arc<Mutex<Node>>, global: Arc<Mutex<GlobalState>>) {
        let my_node = self.get_my_node();
        let server = TcpListener::bind(format!("{}:{}", my_node.ip, my_node.port)).unwrap();
        println!("Listening on {}", server.local_addr().unwrap());
        for stream in server.incoming() {
            self.request(stream.unwrap(), Arc::clone(&tree), Arc::clone(&global));
        }
    }
}
