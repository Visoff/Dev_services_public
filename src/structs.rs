use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ChainComponentPlace {
    pub from: HashMap<String, String>,
    pub into: HashMap<String, String>
}

#[derive(Debug, Clone)]
pub struct ChainComponent {
    pub from: Box<Component>,
    pub to: Box<Component>,
    pub place: Vec<ChainComponentPlace>
}

#[derive(Debug, Clone)]
pub struct ExposedComponent {
    pub host: String,
    pub port: i64
}

#[derive(Debug, Clone)]
pub struct ProxyComponent {
    pub service_name: String
}

#[derive(Debug, Clone)]
pub struct Component {
    pub proxy: Option<ProxyComponent>,
    pub exposed: Option<ExposedComponent>,
    pub chain: Option<ChainComponent>
}

#[derive(Debug)]
pub struct Node {
    pub next: HashMap<String, Node>,
    pub value: Option<Component>
}

#[derive(Debug, Clone)]
pub struct ServiceRun {
    pub way: String,
    pub tag: String
}

#[derive(Debug, Clone)]
pub struct ServiceSets {
    pub headers: HashMap<String, String>,
    pub path: String,
    pub body: serde_json::Value
}

#[derive(Debug, Clone)]
pub struct Service {
    pub server: String,
    pub port: u16,
    pub uses: Vec<String>,
    pub sets: ServiceSets,
    pub run: Option<ServiceRun>
}

impl ChainComponentPlace {
    pub fn new(from:HashMap<String, String>, into:HashMap<String, String>) -> Self {
        return ChainComponentPlace {
            from,
            into,
        };
    }
}

impl Component {
    pub fn new_proxy(service_name: String) -> Self {
        return Component {
            proxy: Some(ProxyComponent {
                service_name
            }),
            exposed: None,
            chain: None
        };
    }
    pub fn new_exposed(host:String, port: i64) -> Self {
        return Component {
            proxy: None,
            exposed: Some(ExposedComponent { host, port }),
            chain: None
        };
    }
    pub fn new_chain(from: Box<Component>, to: Box<Component> , place: Vec<ChainComponentPlace>) -> Self {
        return Component {
            proxy: None,
            exposed: None,
            chain: Some(ChainComponent {from, to, place})
        };
    }
}

impl Node {
    pub fn new() -> Self {
        return Node {
            next: HashMap::new(),
            value: None
        };
    }

    pub fn insert(&mut self, path:String, value:Option<Component>) {
        let parts: Vec<&str> = path.split("/").filter(|el| !el.is_empty()).collect();
        if path.is_empty() {
            self.value = value
        }
        else {
            self.next
                .entry(parts[0].to_string())
                .or_insert_with(Node::new)
                .insert(parts[1..].join("/"), value);
        }
    }
}

impl ServiceSets {
    pub fn new() -> Self {
        return ServiceSets{
            headers: HashMap::new(),
            path: String::new(),
            body: serde_json::Value::String("".to_owned()),
        };
    }
}

impl Service {
    pub fn new(server: String, port: u16, run: Option<ServiceRun>) -> Self {
        return Service{
            server,
            port,
            uses: Vec::new(),
            sets: ServiceSets::new(),
            run
        };
    }
}