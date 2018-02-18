use std::io::Read;

pub struct Files<S: AsRef<str>, R: Read>(pub Vec<(S, R)>);

impl<T, S, R> From<Option<T>> for Files<S, R> where S: AsRef<str>, R: Read,
                                                    T: Into<Files<S, R>> {
    fn from(val: Option<T>) -> Self {
        match val {
            None => Files(vec![]),
            Some(v) => v.into()
        }
    }
}

impl<'a, S, R> From<Vec<(S, R)>> for Files<S, Box<Read + 'a>> where S: AsRef<str>, R: Read + 'a{
    fn from(vec: Vec<(S, R)>) -> Self {
        Files(vec.into_iter().map(|(s,r)| (s, Box::new(r) as Box<Read>)).collect())
    }
}


impl<S: AsRef<str>, R: Read> Files<S, R> {

    pub fn followed_by(mut self, mut other: Self) -> Self {
        self.0.append(&mut other.0);
        self
    }

}

impl<S: AsRef<str>, R: Read> IntoIterator for Files<S, R> {
    type Item = (S, R);
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}