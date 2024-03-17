use clap::{Command, Arg, ArgMatches};

pub fn parse_cli() -> ArgMatches {
    let app = Command::new("DevSync")
        .version("0.0.1")
        .author("DevSync Team")
        .about("DevSync")
        .arg(
            Arg::new("network")
            .short('n')
            .long("network")
        )
        .arg(
            Arg::new("file")
            .short('f')
            .long("file")
        )
        .arg(
            Arg::new("ip")
            .long("ip")
        )
        .arg(
            Arg::new("port")
            .short('p')
            .long("port")
        );
    return app.get_matches();
}
