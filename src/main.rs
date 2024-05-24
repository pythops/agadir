use russh_keys::key::KeyPair;
use std::process::exit;
use tracing::{error, info};

use std::{
    cmp::Reverse,
    env,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use agadir::{cli, formatter::Formatter, post::load_posts, server::AppServer, tracing::Tracing};
use chrono::NaiveDate;
use ratatui::text::Text;

#[tokio::main]
async fn main() {
    Tracing::init().unwrap();

    let matches = cli::cli().get_matches();

    let port = matches.get_one::<u16>("port").unwrap().to_owned();

    let (formatter_config, formatter_assets) = Formatter::init();
    let formatter = Formatter::new(&formatter_config, &formatter_assets);

    let main_dir = {
        match env::var("AGADIR") {
            Ok(v) => PathBuf::from(v),
            Err(_) => dirs::home_dir().unwrap().join(".agadir"),
        }
    };

    if !main_dir.exists() {
        error!(
            "The directory {:?} does not exist. Please create it before continuing",
            main_dir
        );
        exit(1);
    }

    let posts_dir = main_dir.join("posts");

    if !posts_dir.exists() {
        error!(
            "The directory {:?} does not exist. Please create it before continuing",
            posts_dir
        );
        exit(1);
    }

    let posts = load_posts(posts_dir, formatter);

    let mut toc: Vec<(NaiveDate, Text<'static>)> = Vec::new();

    for post in posts.iter() {
        toc.push((post.created_at, post.title.clone()));
    }

    toc.sort_by_key(|k| Reverse(k.0));

    let key_path = main_dir.join("key");

    let key = match File::open(&key_path) {
        Ok(mut f) => {
            info!("{}", format!("Loading the server key {:?}", key_path));
            let buf: [u8; 32] = [0; 32];
            f.read_to_end(&mut buf.to_vec()).unwrap();
            ed25519_dalek::SigningKey::from_bytes(&buf)
        }
        Err(_) => {
            info!("Generating new server key");
            let key_pair = russh_keys::key::KeyPair::generate_ed25519().unwrap();
            let KeyPair::Ed25519(signing_key) = key_pair;
            let file = File::create(key_path).unwrap();
            let mut buffer = std::io::BufWriter::new(file);
            buffer.write_all(&signing_key.to_bytes()).unwrap();
            signing_key
        }
    };

    let mut server = AppServer::new(posts, toc, key);
    server.run(port).await.expect("Failed running server");
}
