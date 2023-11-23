mod structs;
mod http;
mod parsing;
mod services;
mod handle_services;

use parsing::*;
use services::run_service;
use handle_services::*;

fn main() {
    let (mut tree, services) = parse_config("setup.json".to_string()).unwrap();
    println!("{:?}", services);
    println!("{:?}", tree);
    for (_, service) in services.to_owned() {
        run_service(service);
    }
    if let Some(ref root_component) = tree.value {
        if let Some(ref exposed_component) = root_component.exposed {
            start_endpoint_server(exposed_component.host.to_owned(), exposed_component.port.to_string().parse::<u16>().unwrap(), &mut tree, services);
        }
    }
}
