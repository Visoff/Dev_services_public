use std::sync::{Arc, Mutex};
use std::{collections::HashMap, net::TcpStream, io::prelude::*};

use crate::structs::http::{Request, Response, split_path};

pub fn get_ro_from_mutex<T: Clone>(m: &Arc<Mutex<T>>) -> Option<T> {
    if let Ok(data) = m.lock() {
        return Some(data.clone());
    } else {
        return None;
    }
}

#[derive(Clone)]
pub struct Exposed {
    pub host: String,
    pub port: i64
}

impl Exposed {
    pub fn stringify(&self) -> String {
        return format!("{}:{}", self.host, self.port);
    }
    pub fn from_json(value:serde_json::Value) -> Result<Self, String> {
        let mut result = Exposed { host: "0.0.0.0".to_string(), port: 3000 };
        if let Some(host) = value.get("host") {
            if let Some(host) = host.as_str() {
                result.host = host.to_string();
            } else {
                return Err("Host must be string".to_string());
            }
        }
        if let Some(port) = value.get("port") {
            if let Some(port) = port.as_i64() {
                result.port = port;
            } else {
                return Err("Port must be string".to_string());
            }
        }
        return Ok(result);
    }
}

#[derive(Clone)]
pub struct GlobalState {
    pub services: HashMap<String, Service>,
    pub exposed: Option<Exposed>
}

impl GlobalState {
    pub fn blank() -> Self {
        return GlobalState { services: HashMap::new(), exposed: None }
    }
}

#[derive(Clone)]
pub struct Service {
    host: String,
    port: i64
}

impl Service {
    pub fn from_val(val:serde_json::Value) -> Result<Self, String> {
        let data = match val.as_object() {
            Some(data) => data,
            None => return Err("Service must be an object".to_string())
        };
        let host = match data.get("host") {
            Some(host) => match host.as_str() {
                Some(host) => host.to_string(),
                None => return Err("Service host must be type of string".to_string())
            },
            None => "localhost".to_string()
        };
        let port = match data.get("port") {
            Some(port) => match port.as_i64() {
                Some(port) => port,
                None => return Err("Service port must be type of number".to_string())
            },
            None => 3000
        };
        return Ok(Service { host, port });
    }

    pub fn fetch(&self, req: Request) -> Response {
        let con = TcpStream::connect(format!("{}:{}", self.host, self.port));
        let mut con = match con {
            Ok(con) => con,
            Err(_) => return Response { status_code: 500, status: "ServerError".to_string(), headers: HashMap::new(), body: String::new() }
        };
        con.write_all(req.stringify().as_bytes()).unwrap();
        return match Response::from_stream(con) {
            Some(resp) => resp,
            None => Response { status_code: 500, status: "ServerError".to_string(), headers: HashMap::new(), body: String::new() }
        }
    }
}

impl GlobalState {
    pub fn empty() -> Self {
        return GlobalState { services: HashMap::new(), exposed: None }
    }
}

pub trait CloneComponent {
    fn clone_box(&self) -> Box<dyn Component>;
}
impl<T> CloneComponent for T
where
    T: 'static + Component + Clone
{
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Component> {
    fn clone(&self) -> Box<dyn Component> {
        self.clone_box()
    }
}

pub trait Component: CloneComponent {
    fn call(&self, global: &GlobalState, req: Request) -> Response;
    fn parse(val:serde_json::Value) -> Self where Self: Sized;
}

#[derive(Clone)]
pub struct Node {
    pub next: HashMap<String, Node>,
    pub value: Option<Box<dyn Component>>
}

unsafe impl Send for Node {}

impl Node {
    pub fn new() -> Self {
        return Node {
            next: HashMap::new(),
            value: None
        };
    }

    pub fn insert(&mut self, path:String, component:Box<dyn Component>) {
        self.raw_insert(split_path(path), component);
    }

    pub fn raw_insert(&mut self, parts:Vec<String>, component:Box<dyn Component>) {
        if parts.is_empty() {
            self.value = Some(component);
        } else {
            let current:String = parts[0].clone();
            let other_parts:Vec<String> = parts.iter()
                .enumerate()
                .filter(|&(i, _)| i != 0)
                .map(|(_, el)| el.to_string())
                .collect();
            if let Some(ref mut next) = self.next.get_mut(&current) {
                next.raw_insert(other_parts, component);
            } else {
                let mut next = Node::new();
                next.raw_insert(other_parts, component);
                self.next.insert(current, next);
            }
        }
    }

    pub fn search(&self, path:String) -> (&Node, String) {
        return self.raw_search(split_path(path));
    }

    pub fn raw_search(&self, parts:Vec<String>) -> (&Node, String) {
        if parts.is_empty() {
            return (self, "".to_string());
        }
        let current:String = parts[0].clone();
        let other_parts:Vec<String> = parts.iter()
            .enumerate()
            .filter(|&(i, _)| i != 0)
            .map(|(_, el)| el.to_string())
            .collect();
        if let Some(ref next) = self.next.get(&current) {
            return next.raw_search(other_parts);
        }
        return (self, parts.join("/"));
    }
}

