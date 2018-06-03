#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl<'a> Into<String> for &'a Author {

    fn into(self) -> String {
        let name: String = self.name.as_ref().map(|s| s.clone()).unwrap_or("".into());
        let url: String = format!("(https://github.com/{})", self.login);
        let email: String = self.email.as_ref().map(|s| if s == "" { s.clone() } else { format!("<{}>", s) }).unwrap_or("".into());

        let elements: Vec<_> = vec![name, url, email].into_iter().filter(|s| s.len() > 0)
            .collect();
        elements.join(" ")
    }

}

use files::Files;
use std::io::{Read, Cursor};
impl<'a> Into<Files<&'a str, Box<Read>>> for &'a Author {

    fn into(self) -> Files<&'a str, Box<Read>> {
        let s: String = self.into();
        Files(vec![(".authors", Box::new(Cursor::new(s.into_bytes())))])
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use serde_jsondd;

    #[test]
    fn to_string() {
        let author = Author {
            login: "yrashk".into(),
            name: None,
            email: None,
        };
        let s: String = (&author).into();
        assert_eq!(s, "(https://github.com/yrashk)");

        let author = Author {
            login: "yrashk".into(),
            name: Some("Yurii".into()),
            email: None,
        };
        let s: String = (&author).into();
        assert_eq!(s, "Yurii (https://github.com/yrashk)");

        let author = Author {
            login: "yrashk".into(),
            name: Some("Yurii".into()),
            email: Some("foo@bar.com".into()),
        };
        let s: String = (&author).into();
        assert_eq!(s, "Yurii (https://github.com/yrashk) <foo@bar.com>");

        let author = Author {
            login: "yrashk".into(),
            name: None,
            email: Some("foo@bar.com".into()),
        };
        let s: String = (&author).into();
        assert_eq!(s, "(https://github.com/yrashk) <foo@bar.com>");

        let author = Author {
            login: "yrashk".into(),
            name: None,
            email: Some("".into()),
        };
        let s: String = (&author).into();
        assert_eq!(s, "(https://github.com/yrashk)");

        let author = Author {
            login: "yrashk".into(),
            name: Some("Yurii".into()),
            email: None,
        };
        let s: String = (&author).into();
        assert_eq!(s, "Yurii (https://github.com/yrashk)");


    }

}