use std::fs;

use crate::structs::{data::{Component, GlobalState}, http::{Request, Response, merge_paths}};

use std::collections::HashMap;
use libloading::{Library, Symbol};

pub struct ImportedComponents {
    pub components: HashMap<String, Box<dyn Component>>
}

impl ImportedComponents {
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
    pub path: String
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

impl Component for StaticComponent {
    fn parse(val:serde_json::Value) -> Self where Self: Sized {
        if let Some(path) = val.get("path") {
            return StaticComponent { path: path.as_str().unwrap().to_string() }
        }
        return StaticComponent { path: "".to_string() };
    }
    fn call(&self, _global: &GlobalState, req: Request) -> Response {
        if let Ok(file) = fs::read_to_string(merge_paths(self.path.to_owned(), req.uri)) {
            let mut res = Response::new();
            res.body = file;
            return res;
        } else {
            return Response::not_found();
        }

    }
}


