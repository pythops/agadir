use clap::{arg, crate_version, Command};

pub fn cli() -> Command {
    Command::new("agadir")
        .about("Blogging over the terminal")
        .version(crate_version!())
        .arg(
            arg!(--port <port>)
                .short('p')
                .required(false)
                .help("listening port")
                .default_value("2222")
                .value_parser(clap::value_parser!(u16)),
        )
}
