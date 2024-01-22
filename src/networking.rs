use std::{collections::HashMap, net::{TcpListener, TcpStream}, io::{BufReader, BufRead, Write}};

pub struct Transaction {
    from: String,
    method: String,
    params: HashMap<String, String>
}

pub struct Network {
    call_stack: HashMap<String, Transaction>,
    nodes: HashMap<String, NetworkNode>
}

pub struct NetworkNode {
    ip: String,
    port: String
}

pub fn parse_message(stream: &TcpStream) -> (String, String, String, HashMap<String, String>) {
        let mut reader = BufReader::new(stream);
        let mut buffer = String::new();

        reader.read_line(&mut buffer).unwrap();
        let mut data_line = buffer.split_whitespace();
        let id = data_line.next().unwrap();
        let from = data_line.next().unwrap();
        let method = data_line.next().unwrap();
        
        let mut params: HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            line = line.trim().to_string();
            if line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(": ").unwrap();
            params.insert(name.to_string(), value.to_string());
        }
        return (id.to_string(), from.to_string(), method.to_string(), params);
    }

impl Network {
    pub fn new() -> Network {
        Network {
            call_stack: HashMap::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn connect(myself: NetworkNode, ip: String, port: String) -> Network {
        // request connection from node with ip and port and start approve transaction

        // placeholder:
        Network::new()
    }

    pub fn approve(&mut self, id: String) {
        let origin = self.call_stack.get(&id).unwrap();
        let sender = self.nodes.get(&origin.from).unwrap();
        let mut stream = TcpStream::connect(format!("{}:{}", sender.ip, sender.port)).unwrap();
        stream.write("123 myself approved\r\n\r\n".as_bytes()).unwrap();
    }

    pub fn request(&mut self, stream: TcpStream) {
        let (id, from, method, params) = parse_message(&stream);
        match method.as_str() {
            "connect" => {
                let ip = params.get("ip").unwrap();
                let port = params.get("port").unwrap();
                self.nodes.insert(from, NetworkNode { ip: ip.to_string(), port: port.to_string() });
                self.start_transaction(id, "127.0.0.1:1702".to_string(), "approve_connection".to_string(), params);
            },
            &_ => {println!("Unknown method: {}", method)}
        }
    }

    pub fn start_transaction(&mut self, id: String, from: String, method: String, params: HashMap<String, String>) {
        self.call_stack.insert(id.to_string(), Transaction { from, method, params });
        self.approve(id);
    }

    pub fn listen(&mut self) {
        let server = TcpListener::bind(format!("127.0.0.1:1702")).unwrap();
        for stream in server.incoming() {
            self.request(stream.unwrap());
        }
    }
}
