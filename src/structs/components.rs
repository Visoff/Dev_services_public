use std::fs;

use crate::structs::{data::{Component, GlobalState}, http::{Request, Response, merge_paths, path_exists}};

use std::collections::HashMap;
use libloading::{Library, Symbol};

pub struct ImportedComponents {
    pub components: HashMap<String, Box<dyn Component>>
}

impl ImportedComponents {
    pub fn init() -> Self {
        return ImportedComponents { components: HashMap::new() };
    }

    pub fn import_component(&mut self, library_path: &str) {
        unsafe {
            let library = Library::new(library_path).unwrap();
            let func: Symbol<fn() -> (String, Box<dyn Component>)> = library.get(b"export_component").unwrap();
            let (name, component) = func();
            self.components.insert(name, component);
        }
    }
}

pub struct ProxyComponent {
    pub service: String
}

pub struct StaticComponent {
    pub path: String,
    pub index: String
}

impl Component for ProxyComponent {
    fn parse(val:serde_json::Value) -> Self where Self: Sized {
        let service = match val.as_object() {
            Some(component) => match component.get("service") {
                Some(service) => match service.as_str() {
                    Some(service_name) => service_name,
                    None => panic!("Proxy component service name must be type of string")
                },
                None => panic!("Proxy component must have service")
            },
            None => panic!("Error while parsing proxy component")
        };
        return ProxyComponent { service: service.to_string() };
    }
    fn call(&self, global: &GlobalState, req: Request) -> Response {
        return match global.services.get(&self.service) {
            Some(service) => service.fetch(req),
            None => Response::not_found()
        }
    }
}

impl StaticComponent {
    pub fn respond(path: &str) -> Response {
        let mut res = Response::new();
        let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
        let file = fs::read_to_string(&path).unwrap();
        res.headers.insert("Content-Type".to_string(), mime_type.to_string());
        res.body = file;
        return res;
    }
}

impl Component for StaticComponent {
    fn parse(val:serde_json::Value) -> Self where Self: Sized {
        let mut path = "".to_string();
        let mut index = "index.html".to_string();
        if let Some(parse_path) = val.get("path") {
            path = parse_path.as_str().unwrap().to_string();
        }
        if let Some(parse_index) = val.get("index") {
            index = parse_index.as_str().unwrap().to_string();
        }
        return StaticComponent { path, index };
    }
    fn call(&self, _global: &GlobalState, req: Request) -> Response {
        let path = merge_paths(&self.path, &req.uri);
        if path_exists(&path) {
            return StaticComponent::respond(&path);
        } else if path_exists(&merge_paths(&path, &self.index)) {
            return StaticComponent::respond(&merge_paths(&path, &self.index));
        } {
            return Response::not_found();
        }

    }
}


