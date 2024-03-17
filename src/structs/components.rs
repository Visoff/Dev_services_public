use std::fs;

use crate::structs::{data::{Component, GlobalState}, http::{Request, Response, merge_paths, path_exists}};

pub fn global_parse(val:serde_json::Value) -> Box<dyn Component> {
    let component = val.as_object().unwrap();
    let component_type = component.get("type").unwrap().as_str().unwrap();

    match component_type {
        "proxy" => Box::new(ProxyComponent::parse(val)),
        "static" => Box::new(StaticComponent::parse(val)),
        _ => panic!("Unknown component type")
    }
}

#[derive(Clone)]
pub struct ProxyComponent {
    pub service: String
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

#[derive(Clone)]
pub struct StaticComponent {
    pub path: String,
    pub index: String
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

