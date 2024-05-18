use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use chrono::NaiveDate;
use ratatui::text::Text;
use tracing::error;

use crate::formatter::Formatter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Post<'a> {
    pub title: Text<'a>,
    pub created_at: NaiveDate,
    pub modified_at: NaiveDate,
    pub content: Text<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaData {
    pub title: String,
    pub created_at: String,
    pub modified_at: String,
}

pub fn load_posts(path: PathBuf, formatter: Formatter<'_>) -> Vec<Post<'static>> {
    let mut posts: Vec<Post> = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

        if entry.file_name() == "assets" {
            continue;
        }

        let path = entry.path();

        let mut buf = String::new();
        let Ok(mut file) = File::open(path.clone()) else {
            error!("{}", format!("Can not open the file {:?}", path.clone()));
            continue;
        };
        if file.read_to_string(&mut buf).is_err() {
            error!("{}", format!("Can not read the file {:?}", path.clone()));
            continue;
        }

        if let Some(text) = buf.clone().strip_prefix("---\n") {
            if let Some(index) = text.find("---\n") {
                let metadata = &text[..index];
                let file_name = path.file_name().unwrap().to_str().unwrap();

                let metadata = match serde_yaml::from_str::<MetaData>(metadata) {
                    Ok(v) => v,
                    Err(e) => {
                        error!(
                            "{}",
                            format!(
                                "Can not parse the metadata from the file `{}`: {}",
                                file_name, e
                            ),
                        );
                        continue;
                    }
                };

                let title = Text::from(metadata.title);

                let Ok(created_at) = NaiveDate::parse_from_str(&metadata.created_at, "%d/%m/%Y")
                else {
                    error!(
                            "{}",
                            format!("`created_at` field from the file `{}` is not in the right format `%d/%m/%Y`", file_name),
                        );
                    continue;
                };

                let Ok(modified_at) = NaiveDate::parse_from_str(&metadata.modified_at, "%d/%m/%Y")
                else {
                    error!(
                            "{}",
                            format!("`modified_at` field from the file `{}` is not in the right format `%d/%m/%Y`", file_name),
                        );
                    continue;
                };

                let content = formatter.format(&text[index + 5..]);

                let post = Post {
                    title,
                    created_at,
                    modified_at,
                    content,
                };

                posts.push(post.clone());
            }
        }
    }

    posts
}
