use std::{collections::HashMap, fs};

use uuid::Uuid;

use crate::structs::{data::*, components::*};

pub fn parse_root(component:serde_json::Value, global:&mut GlobalState) -> Result<(), String> {
    if let Some(exposed) = component.get("exposed") {
        global.exposed = Some(Exposed::from_json(exposed.clone())?);
    }
    if let Some(services) = component.get("services") {
        parse_services(&mut global.services, services.clone())?;
    }
    Ok(())
}

pub fn parse_requests(comp:serde_json::Value, global:&GlobalState) -> Result<Node, String> {
    let mut res = Node::new();
    if let Some(requests) = comp.as_array() {
        for request in requests {
            let (uri, comp) = parse_request(request.clone(), global)?;
            res.insert(uri, comp);
        }
    }
    Ok(res)
}

pub fn parse_request(comp:serde_json::Value, global:&GlobalState) -> Result<(String, Box<dyn Component>), String> {
    let uri: String = match comp.get("uri") {
        Some(uri) => match uri.as_str() {
            Some(uri) => uri.to_string(),
            None => return Err("Uri must be string".to_string())
        },
        None => return Err("Every request must have uri".to_string())
    };
    if let Some(name) = comp.get("name") {
        if let Some(name) = name.as_str() {
            let _component = match global.services.get(name) {
                Some(component) => component,
                None => return Err("Trying to use non declared service".to_string())
            };
            //return Ok((uri, *component));
        } else {
            return Err("Name must be string".to_string());
        }
    }
    return Ok((uri, parse_component(comp)?));
}

pub fn parse_services(services:&mut HashMap<String, Service>, val:serde_json::Value) -> Result<(), String> {
    if let Some(service_array) = val.as_array() {
        for serv in service_array {
            let (name, service) = parse_service(serv.clone())?;
            services.insert(name, service);
        }
    };
    Ok(())
}

pub fn parse_service(val:serde_json::Value) -> Result<(String, Service), String> {
    let service = match val.as_object() {
        Some(c) => c,
        None => return Err("Service must be an object".to_string())
    };
    let name:String = match service.get("name") {
        Some(name) => match name.as_str() {
            Some(name) => name.to_string(),
            None => return Err("Service name must be a string".to_string())
        },
        None => Uuid::new_v4().to_string()
    };
    return Ok((name, Service::from_val(val)?));
}

pub fn parse_component(val:serde_json::Value) -> Result<Box<dyn Component>, String> {
    let component = match val.as_object() {
        Some(component) => component,
        None => return Err("Каждый компонент долен быть обьектом".to_string())
    };
    let component_type = match component.get("type") {
        Some(component_type) => component_type.as_str().unwrap(),
        None => return Err("В компоненте не найден type".to_string())
    };
    
    return match component_type {
        "static" => Ok(Box::new(StaticComponent::parse(val))),
        "proxy" => Ok(Box::new(ProxyComponent::parse(val))),
        &_ => Err("Invalid component type".to_string())
    };
}

pub fn parse_config(file_path: String) -> Result<(Node, GlobalState), String> {
    let config = fs::read_to_string(&file_path);
    let config = match config {
        Ok(config) => config,
        Err(_) => {
            return Err(format!("File {} does not exist", file_path));
        }
    };
    let config: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&config);
    let config = match config {
        Ok(config) => config,
        Err(_) => {
            return Err(format!("File {} does not seem to be json formated", file_path));
        }
    };
    
    let mut global: GlobalState = GlobalState::empty();
    parse_root(config.clone(), &mut global)?;
    let tree = match config.get("requests") {
        Some(req) => parse_requests(req.clone(), &mut global)?,
        None => Node::new()
    };
    Ok((tree, global))
}
