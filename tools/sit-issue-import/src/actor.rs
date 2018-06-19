use author::Author;
#[derive(Deserialize, Debug)]
#[serde(tag = "__typename")]
pub enum Actor {
    User(Author),
    #[serde(rename_all = "camelCase")]
    Bot {
        login: String
    }
}

impl<'a> Into<String> for &'a Actor {

    fn into(self) -> String {
        match self {
            &Actor::User(ref author) => author.into(),
            &Actor::Bot { ref login } => format!("https://github.com/{}", login),
        }
    }

}

use files::Files;
use std::io::{Read, Cursor};
impl<'a> Into<Files<&'a str, Box<Read>>> for &'a Actor {

    fn into(self) -> Files<&'a str, Box<Read>> {
        let s: String = self.into();
        Files(vec![(".authors", Box::new(Cursor::new(s.into_bytes())))])
    }

}
