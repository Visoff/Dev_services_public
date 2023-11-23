use std::{collections::HashMap, io::{BufReader, Read, BufRead}, net};

pub fn build_request(method: String, path: String, headers:HashMap<String, String>, body: String) -> String{
    let mut req = String::new();
    req += &method;
    req += " /";
    req += &path.split("/").map(|el| el.to_owned()).collect::<Vec<String>>().join("/");
    req += " HTTP/1.1\r\n";
    req += &headers.iter()
        .map(|(name, value)| name.to_owned()+": "+value)
        .collect::<Vec<String>>()
        .join("\r\n");
    req += "\r\n\r\n";
    req += &body;
    return req;
}

pub fn parse_request(mut stream: &mut net::TcpStream) -> (String, String, HashMap<String, String>, String) {
    let mut buf_reader = BufReader::new(&mut stream);
    let mut http_data = String::new();
    let mut headers: HashMap<String, String> = HashMap::new();

    for (i, line) in buf_reader.by_ref().lines().enumerate() {
        let line = line.unwrap();
        if i == 0 {
            http_data = line.to_owned();
            continue;
        }
        if line.is_empty() {
            break;
        }
        let (key, value) = line.split_once(": ").unwrap();
        headers.insert(key.to_owned(), value.to_owned());
    };

    if http_data.is_empty() {
        return (String::new(), String::new(), HashMap::new(), String::new());
    };

    let mut http_data = http_data.split_whitespace();
    let method = http_data.next().unwrap().to_owned();
    let uri = http_data.next().unwrap().to_owned();
    http_data.next().unwrap();
    let mut body: String = String::new();

    if let Some(cl) = headers.get("Content-Length") {
        buf_reader.take(cl.parse::<u64>().unwrap()).read_to_string(&mut body).unwrap();
    }
    return (method, uri, headers, body);
}

pub fn build_response(status_code: usize, status: String, headers:HashMap<String, String>, body: String) -> String {
    let mut res: String = "HTTP/1.1 ".to_owned();
    res += &status_code.to_string();
    res += " ";
    res += &status;
    res +="\r\n";

    res += &headers.into_iter()
        .map(|(key, value)| (key+": "+&value))
        .collect::<Vec<String>>()
        .join("\r\n");

    res += "\r\n\r\n";
    res += &body;

    return res;
}

pub fn parse_response(response: String) -> (bool, HashMap<String, String>, String) {
    let mut i = response.lines();
    let status = i.next().unwrap().split_whitespace().map(|el| el.to_string()).collect::<Vec<String>>()[1].parse::<usize>().unwrap();
    if (status - status % 100) / 100 != 2 {
        return (false, HashMap::new(), String::new());
    }
    let mut headers: HashMap<String, String> = HashMap::new();
    for line in i.by_ref() {
        if line.is_empty() {
            break;
        }
        let (key, value) = line.split_once(": ").unwrap();
        headers.insert(key.to_string(), value.to_string());
    }
    let body = i.by_ref()
        .take_while(|el| !el.is_empty())
        .map(|el| el.to_string())
        .collect::<Vec<String>>()
        .join("\r\n");
    return (true, headers, body);
}