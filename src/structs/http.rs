use std::{net::TcpStream, collections::HashMap, io::{BufReader, prelude::*}};


pub fn split_path(path:String) -> Vec<String> {
    return path
        .split("/")
        .filter(|el| !el.is_empty())
        .map(|el| el.to_string())
        .collect::<Vec<String>>();
}

pub fn merge_paths(a: String, b:String) -> String {
    let mut res: Vec<String> = Vec::new();
    a.split("/").filter(|el| !el.is_empty()).for_each(|el| {
        res.push(el.to_string());
    });
    b.split("/").filter(|el| !el.is_empty()).for_each(|el| {
        res.push(el.to_string());
    });
    return res.join("/");
}

pub struct Request {
    pub uri: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Request {
    pub fn from_stream(mut stream:&TcpStream) -> Option<Self> {
        let mut headers:HashMap<String, String> = HashMap::new();

        let mut buf = BufReader::new(&mut stream);
        let mut data: String = String::new();
        buf.read_line(&mut data).unwrap();

        if data.trim().is_empty() {
            return None;
        }

        loop {
            let mut line:String = String::new();
            buf.read_line(&mut line).unwrap();

            if line.trim().is_empty() {
                break;
            }
            let (key, value) = line.trim().split_once(": ").unwrap();
            headers.insert(key.to_string(), value.to_string());
        }
        
        let mut body: String = String::new();
        let content_length: u64 = headers
            .get("Content-Length")
            .unwrap_or(&"0".to_string())
            .parse().unwrap();
        buf.take(content_length).read_to_string(&mut body).unwrap();
        
        let mut data = data.split_whitespace();
        let method = data.nth(0).unwrap_or_default().to_string();
        let uri = data.nth(0).unwrap_or_default().to_string();

        return Some(Request {
            uri: uri.to_string(),
            headers,
            body,
            method: method.to_string()
        });
    }

    pub fn stringify(&self) -> String {
        let mut res = String::new();
        res+=&self.method;
        res+=" /";
        res+=&self.uri;
        res+=" ";
        res+="HTTP/1.1\r\n";
        self.headers.iter().for_each(|(key, value)| {
            res+=key;
            res+=": ";
            res+=value;
            res+="\r\n";
        });
        res+="\r\n";
        res+=&self.body;
        res+="\r\n";
        return res;
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: String,
    pub status_code: i64,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl Response {
    pub fn new() -> Self {
        return Response {status_code: 200, status: "Ok".to_string(), headers: HashMap::new(), body: String::new()}
    }

    pub fn not_found() -> Self {
        return Response { status_code: 404, status: "NotFound".to_string(), headers: HashMap::new(), body: "404 Not found".to_string() }
    }

    pub fn into_string(&self) -> String {
        let res = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n{}\r\n",
            self.status_code,
            self.status,
            self.headers
                .iter()
                .map(|(key, value)| format!("{}: {}", key, value))
                .collect::<Vec<String>>()
                .join("\r\n"),
            self.body
        );
        return res;
    }

    pub fn from_stream(mut stream:TcpStream) -> Option<Self> {
        let mut buf = BufReader::new(&mut stream);
        let mut data = String::new();
        match buf.read_line(&mut data) {
            Ok(_) => {},
            Err(_) => return None
        };

        let mut data = data.trim().split_whitespace();
        let status_code = data.nth(1).unwrap();
        let status = data.nth(0).unwrap();
        let mut headers:HashMap<String, String> = HashMap::new();
        loop {
            let mut line = String::new();
            buf.read_line(&mut line).unwrap();
            let line = line.trim();

            if line.is_empty() {
                break;
            }
            let (key, value) = line.split_once(": ").unwrap();
            headers.insert(key.to_string(), value.to_string());
        }
        let mut body = String::new();
        let content_length: u64 = headers
            .get("Content-Length")
            .unwrap_or(&"0".to_string())
            .parse()
            .unwrap();
        buf.take(content_length).read_to_string(&mut body).unwrap();

        return Some(Response {
            status_code: status_code.parse().unwrap(),
            status: status.to_string(),
            headers,
            body
        });
    }
}


