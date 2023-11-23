use std::{collections::HashMap, process::Command};

use uuid::Uuid;

use crate::structs::*;


pub fn add_service(service_component:&serde_json::Value, services:&mut HashMap<String, Service>) -> String {
    let service_component = match service_component.as_object() {
        Some(service_component) => service_component,
        None => todo!()
    };
    let service_name = match service_component.get("name") {
        Some(service_name) => service_name.as_str().to_owned().unwrap().to_string(),
        None => Uuid::new_v4().to_string()
    };
    let service_server = match service_component.get("server") {
        Some(service_server) => match service_server.as_str().unwrap() {
            "self" => "127.0.0.1",
            &_ => service_server.as_str().unwrap()
        },
        None => todo!()
    };
    let service_port = match service_component.get("port") {
        Some(service_port) => service_port.as_u64().unwrap(),
        None => 80
    };
    let service_run = match service_component.get("run") {
        Some(service_run) => service_run.as_object().unwrap().to_owned(),
        None => serde_json::Map::new()        
    };
    let way = service_run.keys().collect::<Vec<&String>>();
    let way = way.last();
    let service_run = match way {
        Some(way) => Some(ServiceRun{way: way.to_string().to_owned(), tag: service_run.get(&way.to_string().to_owned()).unwrap().as_str().unwrap().to_string()}),
        None => None
    };
    let mut service: Service = Service::new(service_server.to_string(), service_port as u16, service_run);

    service.uses = match service_component.get("use") {
        Some(service_use) => service_use.as_array().unwrap().iter().map(|el| el.as_str().unwrap().to_string()).collect::<Vec<String>>(),
        None => Vec::new()
    };

    let service_sets = match service_component.get("set") {
        Some(service_set) => service_set.as_object().unwrap().to_owned(),
        None => serde_json::Map::new()
    };

    service.sets.path = match service_sets.get("url") {
        Some(path) => path.as_str().unwrap().to_string(),
        None => String::new()        
    };
    service.sets.body = match service_sets.get("body") {
        Some(body) => body.to_owned(),
        None => serde_json::Value::String("".to_string())        
    };
    service.sets.headers = match service_sets.get("headers") {
        Some(headers) => headers.as_object().unwrap().iter()
            .map(|(key, value)| (key.to_owned(), value.as_str().unwrap().to_string()))
            .collect(),
        None => HashMap::new()
    };

    services.insert(service_name.to_owned(), service);
    return service_name;
}

pub fn run_service(service: Service) {
    if service.run.is_none() {
        return;
    }
    let run = service.run.unwrap();
    println!("{:?}", run);
    if run.way.is_empty() {
        return;
    }
    if run.way == "image" {
        let cmd = ["docker", "run", "-dp", &(service.port.to_string()+":"+&service.port.to_string()), "-t", &run.tag];
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .arg("/C")
                .args(cmd)
                .output()
                .expect("Error while executing command")
        } else {
            Command::new("sh")
                .arg("-C")
                .args(cmd)
                .output()
                .expect("Error while executing command")
        };
        println!("{}: {}", run.tag, String::from_utf8_lossy(&output.stdout));
    }
}