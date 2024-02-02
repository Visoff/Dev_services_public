use std::{collections::HashMap, net::{TcpListener, TcpStream}, io::{BufReader, BufRead, Write, Read}};


#[derive(Clone)]
pub struct Message {
    id: String,
    from: String,
    method: String,
    params: HashMap<String, String>,
    content: String
}

impl Message {
    pub fn build(from: String, method: String, params: HashMap<String, String>, content: String) -> Self {
        Message {
            id: uuid::Uuid::new_v4().to_string(),
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
        let id = data_line.next().unwrap();
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
            id: id.to_string(),
            from: from.to_string(),
            method: method.to_string(),
            params,
            content
        };
    }

    pub fn as_string(&self) -> String {
        let mut message = String::new();
        message.push_str(&self.id);
        message.push_str(" ");
        message.push_str(&self.from);
        message.push_str(" ");
        message.push_str(&self.method);
        for (k, v) in &self.params {
            message.push_str("\r\n");
            message.push_str(&format!("{}: {}", k, v));
        }
        message.push_str("\r\n");
        message.push_str(&format!("Content-Length: {}", self.content.len()));
        message.push_str("\r\n\r\n");
        message.push_str(&self.content);
        message
    }

    pub fn send(&self, receiver: &NetworkNode) {
        let mut stream = TcpStream::connect(format!("{}:{}", receiver.ip, receiver.port)).unwrap();
        stream.write(self.as_string().as_bytes()).unwrap();
    }

    pub fn is_transaction(&self) -> bool {
        self.params.get("transaction_id").is_some()
    }

    pub fn build_transaction(&self, net: &Network) -> Transaction {
        Transaction::new(self.clone(), net.nodes.keys().cloned().collect())
    }

    pub fn build_transaction_approval(&self, from: String) -> Message {
        let mut params = HashMap::new();
        params.insert("transaction_id".to_string(), self.params.get("transaction_id").unwrap().to_string());
        Message::build(from, "approve_transaction".to_string(), params, "".to_string())
    }
}

pub struct Transaction {
    message: Message,
    aproved: HashMap<String, bool>
}

impl Transaction {
    pub fn new(message: Message, nodes: Vec<String>) -> Transaction {
        Transaction {
            message,
            aproved: nodes.iter().map(|n| (n.to_string(), false)).collect()
        }
    }

    pub fn get_message(&self) -> &Message {
        &self.message
    }

    pub fn is_approved(&self) -> bool {
        self.aproved.values().all(|v| *v)
    }

    pub fn send_for_approval(&self, net: &Network) {
        for (id, _) in &self.aproved {
            self.message.build_transaction_approval(id.to_string()).send(&net.nodes.get(id).unwrap());
        }
    }
}

pub struct Network {
    call_stack: HashMap<String, Transaction>,
    nodes: HashMap<String, NetworkNode>,
    my_id: String
}

#[derive(Debug)]
pub struct NetworkNode {
    ip: String,
    port: String,
}

impl Network {
    pub fn new(ip: String, port: String) -> Network {
        let mut network = Network {
            call_stack: HashMap::new(),
            nodes: HashMap::new(),
            my_id: uuid::Uuid::new_v4().to_string()
        };
        network.nodes.insert(network.my_id.to_string(), NetworkNode { ip, port });
        network
    }

    pub fn get_my_node(&self) -> &NetworkNode {
        self.nodes.get(&self.my_id).unwrap()
    }

    pub fn connect(&mut self, ip: String, port: String) {
        let my_node = self.get_my_node();
        let mut stream = TcpStream::connect(format!("{}:{}", ip, port)).unwrap();
        stream.write(
            format!("{} {} connect\r\nip: {}\r\nport: {}\r\n\r\n",
                    uuid::Uuid::new_v4().to_string(),
                    self.my_id,
                    my_node.ip,
                    my_node.port
                ).as_bytes()).unwrap();
        
    }

    pub fn shout(&self, message: Message) {
        self.nodes.iter().for_each(|(_, node)| {
            message.send(node);
        });
    }

    pub fn request(&mut self, stream: TcpStream) {
        let message = Message::parse(&stream);
        match message.method.as_str() {
            "connect" => {
                println!("Connection request");
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
                println!("{:?}", self.nodes);
            },
            "add_member" => {
                let ip = message.params.get("ip").unwrap().to_string();
                let port = message.params.get("port").unwrap().to_string();
                let name = message.params.get("name").unwrap().to_string();
                self.nodes.insert(name, NetworkNode{ ip, port });
                println!("Added member! New nodes:\n{:?}", self.nodes);
            },
            "add_members" => {
                message.content.split("\n").for_each(|line| {
                    let (name, data) = line.split_once(" ").unwrap();
                    let (ip, port) = data.split_once(":").unwrap();
                    self.nodes.insert(name.to_string(), NetworkNode{ ip: ip.to_string(), port: port.to_string() });
                });
                println!("Added members! Neo nodes:\n{:?}", self.nodes);
            },
            "approve_transaction" => {
                println!("Approve transaction requested from {}", message.from);
                if !message.is_transaction() {
                    return;
                }
            },
            &_ => {
                println!("Unknown method: {}", message.method);
            }
        }
    }

    pub fn listen(&mut self) {
        let my_node = self.get_my_node();
        let server = TcpListener::bind(format!("{}:{}", my_node.ip, my_node.port)).unwrap();
        for stream in server.incoming() {
            self.request(stream.unwrap());
        }
    }
}
