use std::{collections::HashMap, fs};

use crate::{structs::*, services::*};

#[derive(Debug)]
pub enum ParseError {
    ParseConfigError,
    FileNotFoundError,
    ParseFileError
}

pub fn parse_component(component:serde_json::Value, tree:&mut Node, services:&mut HashMap<String, Service>) -> Result<Option<Component>, ParseError> {
    let component = match component.as_object() {
        Some(component) => component,
        None => return Err(ParseError::ParseConfigError {  }) // add error here
    };
    let component_type = match component.get("type") {
        Some(component_type) => component_type.as_str().unwrap(),
        None => return Err(ParseError::ParseConfigError {  }) // add error here
    };
    match component_type {
        "proxy" => {
            let uri_opt = component.get("uri");

            let service = component.get("service");
            let service = match service {
                Some(service) => service,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let service_name: String;
            if service.as_str() != None {
                service_name = service.as_str().unwrap().to_owned();
            } else {
                service_name = add_service(service, services);
            }

            let comp = Component::new_proxy(service_name);

            if let Some(uri) = uri_opt {
                println!("{:?}", uri);
                tree.insert(
                    uri.as_str().unwrap().to_owned(),
                    Some(comp.to_owned())
                );
            }
            Ok(Some(comp))
        },
        "chain" => {
            let uri_opt = component.get("uri");

            let from_val = match component.get("from") {
                Some(from) => from,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let from = parse_component(from_val.to_owned(), tree, services)?;
            if from.is_none() {
                return Err(ParseError::ParseConfigError {  }) // add error here
            }
            let from = from.unwrap();
            
            let to_val = match component.get("into") {
                Some(to) => to,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let to = parse_component(to_val.to_owned(), tree, services)?;
            if to.is_none() {
                return Err(ParseError::ParseConfigError {  }) // add error here
            }
            let to = to.unwrap();

            let place_obj = match component.get("place") {
                Some(place_obj) => place_obj.as_array().unwrap(),
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let place = place_obj.iter()
                .map(|el| el.as_object().unwrap())
                .map(|el| ChainComponentPlace::new(
                    el.get("from").unwrap().as_object().unwrap().iter()
                        .map(|(key, value)| (key.to_string(), value.as_str().unwrap().to_string()))
                        .collect::<HashMap<String, String>>(),
                        el.get("into").unwrap().as_object().unwrap().iter()
                        .map(|(key, value)| (key.to_string(), value.as_str().unwrap().to_string()))
                        .collect::<HashMap<String, String>>()
                    )
                )
                .collect::<Vec<ChainComponentPlace>>();

            let comp = Component::new_chain(Box::new(from), Box::new(to), place);

            if let Some(uri) = uri_opt {
                println!("{:?}", uri);
                tree.insert(
                    uri.as_str().unwrap().to_owned(),
                    Some(comp.to_owned())
                );
            }
            Ok(Some(comp))
        },
        "endpoint" => {
            let data = match component.get("exposed") {
                Some(data) => data,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let data = match data.as_object() {
                Some(data) => data,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let host = match data.get("host") {
                Some(host) => host,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let port = match data.get("port") {
                Some(port) => port,
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            tree.value = Some(Component::new_exposed(host.as_str().unwrap().to_string(), port.as_i64().unwrap()));
            let req = component.get("requests");
            let req = match req {
                Some(req) => match req.as_array() {
                    Some(req) => req,
                    None => return Err(ParseError::ParseConfigError {  }) // add error here
                },
                None => return Err(ParseError::ParseConfigError {  }) // add error here
            };
            let service_components = component.get("services");
            if service_components != None {
                let service_components = match service_components.unwrap().as_array() {
                    Some(service_components) => service_components,
                    None => return Err(ParseError::ParseConfigError {  }) // add error here
                };
                for service_component in service_components {
                    add_service(service_component, services);
                }
            }
            for request in req {
                parse_component(request.to_owned(), tree, services)?;
            }
            Ok(None)
        }
        &_ => return Err(ParseError::ParseConfigError {  }) // add error here
    }
}

pub fn parse_config(file_path: String) -> Result<(Node, HashMap<String, Service>), ParseError> {
    let config = fs::read_to_string(&file_path);
    let config = match config {
        Ok(config) => config,
        Err(_) => {
            println!("File {} does not exist", file_path);
            return Err(ParseError::FileNotFoundError)
        }
    };
    let config: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&config);
    let config = match config {
        Ok(config) => config,
        Err(_) => {
            println!("File {} does not seem to be json formated", file_path);
            return Err(ParseError::ParseFileError)
        }
    };
    
    let mut tree = Node::new();
    let mut services: HashMap<String, Service> = HashMap::new();
    parse_component(config.to_owned(), &mut tree, &mut services)?;
    Ok((tree, services))
}