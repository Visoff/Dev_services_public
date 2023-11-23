use std::{collections::HashMap, net::{TcpStream, self}, io::{Write, BufReader, BufRead}, thread};

use crate::{structs::*, http::*};

pub fn build_request_url(service:&Service) -> String {
    let mut uri = String::new();
    uri += &service.server;
    uri += ":";
    uri += &service.port.to_string();
    return uri
}

pub fn handle_proxy(service:&Service, method: String, remained_uri: String, headers:HashMap<String, String>, body: String) -> String {
    let mut remained_uri = remained_uri.to_owned();
    let mut headers = headers.to_owned();
    let mut body = body.to_owned();
    if !service.uses.contains(&"url".to_string()) {
        remained_uri = service.sets.path
            .split("/")
            .map(|el| el.to_string())
            .filter(|el| !el.is_empty())
            .collect::<Vec<String>>()
            .join("/");
    } else {
        remained_uri = remained_uri.split("/")
            .chain(service.sets.path.split("/"))
            .filter(|el| !el.is_empty())
            .map(|el| el.to_string())
            .collect::<Vec<String>>()
            .join("/");
    }
    if !service.uses.contains(&"headers".to_string()) {
        let allowed = vec![
            "Connection".to_string(),
            "Host".to_string(),
            "Accept".to_string(),
            "Content-Type".to_string()
        ];
        headers = headers.iter()
            .map(|(key, value)| (key.to_owned(), value.to_owned()))
            .filter(|(key, _)| allowed.contains(key))
            .collect();
    }
    if !service.uses.contains(&"body".to_string()) {
        body = serde_json::to_string(&service.sets.body).unwrap();
    } else if Some(&"application/json".to_string()) == headers.get("Content-Type") {
        let parsed_body: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(&body);
        let parsed_body = parsed_body.unwrap();
        body = serde_json::to_string(&merge_body(parsed_body.to_owned(), service.sets.body.to_owned())).unwrap();
    }
    for (name, value) in service.sets.headers.to_owned() {
        headers.insert(name, value);
    }
    
    
    let mut client = TcpStream::connect(build_request_url(service)).unwrap();

    client.write_all(build_request(method, remained_uri, headers, body).as_bytes()).unwrap();
    let reader = BufReader::new(&mut client);
    let res = reader
        .lines();
    let mut lines:Vec<String> = Vec::new();
    let mut body_parsing = false;
    for line in res {
        let line = line.unwrap();
        if line.is_empty() {
            if body_parsing {
                break;
            }
            body_parsing = true;
        }
        lines.push(line);
    }
    let res = lines.join("\r\n")+"\r\n\r\n";
    
    return res;
}

pub fn merge_body(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    if let (serde_json::Value::Object(a), serde_json::Value::Object(b)) = (a.to_owned(), b.to_owned()) {
        let mut merged = a.clone();
        for (key, value) in b {
            if merged.contains_key(&key) {
                merged.insert(key.to_owned(), merge_body(merged.get(&key.to_owned()).unwrap().to_owned(), value.to_owned()));
                continue;
            }
            merged.insert(key, value);
        }
        return serde_json::Value::Object(merged);
    } else if let (serde_json::Value::Array(a), serde_json::Value::Array(b)) = (a.to_owned(), b.to_owned()) {
        let mut merged = a.clone();
        for val in b {
            merged.push(val);
        }
        return serde_json::Value::Array(merged);
    } else {
        return a.to_owned();
    }
}

pub fn get_value_in_object(object: serde_json::Value, path_parts: String) -> serde_json::Value {
    let parts = path_parts.split(".").map(|el| el.to_string()).collect::<Vec<String>>();
    let mut res = object.clone();
    for part in parts {
        let mut part = part;
        let mut indexes: Vec<usize> = Vec::new();
        while part.ends_with("]") {
            part.pop();
            let prt = part.to_owned();
            let (new_part, i) = prt.split_at(prt.rfind("[").unwrap());
            part = new_part.to_string();
            indexes.push(i[1..i.len()].parse::<usize>().unwrap());
        }
        if let Some(obj) = res.get(part) {
            res = obj.to_owned();
        }
        indexes.reverse();
        for index in indexes {
            if let Some(arr) = res.as_array() {
                if let Some(val) = arr.get(index) {
                    res = val.to_owned();
                }
            }
        }
    }
    return res;
}

pub fn handle_component_request(services:HashMap<String, Service>, component: &Component, method: String, remained_uri: String, headers:HashMap<String, String>, body: String, mut stores: HashMap<String, String>) -> String {
    if let Some(proxy_component) = &component.proxy {
        let service = services.get(&proxy_component.service_name).unwrap();
        let resp = handle_proxy(service, method, remained_uri, headers, body.to_owned());
        return resp;
    } else if let Some(chain_component) = &component.chain {
        let from_response =  handle_component_request(services.to_owned(), &chain_component.from, method.to_owned(), remained_uri.to_owned(), headers.to_owned(), body.to_owned(), stores.to_owned());
        let (ok, resp_headers, resp_body) = parse_response(from_response);
        if !ok {
            todo!();
        }
        let mut new_headers: HashMap<String, String> = headers.clone();
        let mut new_body: serde_json::Value = serde_json::from_str(&resp_body.to_owned()).unwrap();
        for rule in &chain_component.place {
            let mut value: String = "undefined".to_string();
            if let Some(header) = rule.from.get("header") {
                if resp_headers.contains_key(header) {
                    value = resp_headers.get(header).unwrap().to_string();
                }
            }
            if let Some(body_path) = rule.from.get("body") {
                value = serde_json::to_string(&get_value_in_object(new_body.to_owned(), body_path.to_string())).unwrap();
            }
            if let Some(store_key) = rule.from.get("store") {
                if let Some(val) = stores.get(store_key) {
                    value = val.to_string();
                }
            }

            if let Some(header) = rule.into.get("header") {
                new_headers.insert(header.to_string(), value.to_owned());
            }
            if let Some(_body_path) = rule.into.get("body") {
                new_body = serde_json::Value::String(value.to_owned());
            }
            if let Some(store_key) = rule.into.get("store") {
                stores.insert(store_key.to_string(), value.to_owned());
            }
        }
        let new_body = serde_json::to_string(&new_body).unwrap();
        return handle_component_request(services, &chain_component.to, method, remained_uri, new_headers, new_body, stores);
    } else {
        return String::new();
    }
}

pub fn start_endpoint_server(host:String, port:u16, tree:&mut Node, services:HashMap<String, Service>) {
    let http = net::TcpListener::bind(host+":"+&port.to_string());
    let http = match http {
        Ok(http) => http,
        Err(_) => todo!()
    };

    thread::scope(|s| {
        for stream in http.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(_) => todo!()
        };
    
        let (method, uri, headers, body) = parse_request(&mut stream);

        if method.is_empty() {
            continue;
        }
        
        let uri_parts = uri.split("/").into_iter();
        let mut endpoint: &Node = tree;
        let mut remained_uri: Vec<String> = Vec::new();
        for part in uri_parts {
                if part.is_empty() {
                    continue;
                }
                endpoint = match endpoint.next.get(part) {
                    Some(endpoint) => endpoint,
                    None => {
                        remained_uri.push(part.to_owned());
                        continue;
                    }
                }
            }
            let remained_uri = remained_uri.join("/");

            if endpoint.value.is_none() {
                stream.write_all(build_response(404, "Not Found".to_string(), HashMap::new(), "".to_string()).as_bytes()).unwrap();
                continue;
            }

            let component = endpoint.value.as_ref().unwrap();

            let services = services.clone();
            s.spawn(move || {
                let resp = handle_component_request(services, component, method, remained_uri, headers, body, HashMap::new());
                stream.write_all(resp.as_bytes()).unwrap();
            });
        }
    });
}