use serde::Deserialize;

pub trait Query {
    type Result;
    fn query(&self) -> String;
}

pub trait Pageable {
    type Item;
    fn after(&self, cursor: String) -> Self;
}

pub trait HasPageInfo {
    fn page_info(&self) -> &PageInfo;
}

pub trait HasTotalCount {
    fn total_count(&self) -> usize;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    has_next_page: bool,
    end_cursor: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Paged<T> {
    pub page_info: PageInfo,
    pub nodes: Vec<T>,
    pub total_count: usize,
}

impl<T> IntoIterator for Paged<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

use reqwest;

enum Maybe<T> {
    None,
    Maybe,
    Some(T),
}

impl<T> From<Option<T>> for Maybe<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            None => Maybe::None,
            Some(v) => Maybe::Some(v),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Position {
    index: u8,
    end_cursor: Option<String>,
}

pub trait PageHandler {
    fn page_position(&self) -> Position;
    fn set_page_position(&mut self, pos: &Position);
}

#[derive(Default)]
pub struct MemoryPageHandler {
    position: Position,
}

impl PageHandler for MemoryPageHandler {
    fn page_position(&self) -> Position {
        self.position.clone()
    }

    fn set_page_position(&mut self, pos: &Position) {
        self.position = pos.clone();
    }
}

use std::str::FromStr;

use std::fmt::Debug;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DataWrapper<T> where T : Debug {
    data: Option<T>,
}

use std::collections::{VecDeque, HashMap};

pub struct PageableRequest<S, Q, P> where S: AsRef<str>, Q : Pageable + Query, P : PageHandler {
    url: S,
    token: S,
    query: Q,
    queued: VecDeque<Q::Item>,
    next_page: Maybe<String>,
    client: reqwest::Client,
    page_handler: P,
    total_count: Option<usize>,
}

impl<S, Q, P> PageableRequest<S, Q, P> where S: AsRef<str>, Q : Pageable + Query, P : PageHandler {
    pub fn new(url: S, token: S, query: Q, page_handler: P) -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        PageableRequest{  url, query, token, queued: VecDeque::new(), next_page: Maybe::Maybe, client, page_handler, total_count: None }
    }
}

impl<S, Q, P, T, Iter> PageableRequest<S, Q, P> where S: AsRef<str>,
                                                      Q : Pageable + Query<Result = T>,
                                                      P: PageHandler,
                                                      Iter : Iterator<Item=Q::Item>,
                                                      T : HasPageInfo + HasTotalCount + IntoIterator<Item=Q::Item, IntoIter=Iter> + Debug {

    pub fn total_count(&self) -> Option<usize> {
        self.total_count.clone()
    }

}

impl<S, Q, P, T, Iter> Iterator for PageableRequest<S, Q, P> where S: AsRef<str>,
                                                                   Q : Pageable + Query<Result = T>,
                                                                   P: PageHandler,
                                                                   Iter : Iterator<Item=Q::Item>,
                                                                   T : HasPageInfo + HasTotalCount + IntoIterator<Item=Q::Item, IntoIter=Iter> + Debug,
                                                                   for<'de> T : Deserialize<'de> {
    type Item = Q::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pos = self.page_handler.page_position();
        match self.queued.pop_front() {
            Some(q) => {
                pos.index += 1;
                self.page_handler.set_page_position(&pos);
                Some(q)
            },
            None => {
                match self.next_page {
                    Maybe::None => return None,
                    Maybe::Maybe => (),
                    Maybe::Some(ref cur) => {
                        pos.index = 0;
                        pos.end_cursor = Some(cur.clone());
                        self.page_handler.set_page_position(&pos);
                        self.query = self.query.after(cur.clone());
                    }
                }
                self.page_handler.set_page_position(&pos);
                let mut map = HashMap::new();
                map.insert("query", self.query.query());
                match self.client.execute(self.client
                    .post(self.url.as_ref())
                    .header(reqwest::header::Authorization(reqwest::header::Bearer::from_str(self.token.as_ref()).unwrap()))
                    .json(&map)
                    .build()
                    .unwrap()) {
                    Err(e) => panic!("{}", e),
                    Ok(mut resp) => {
                        let response : DataWrapper<T> = resp.json().unwrap();
                        let result = response.data.unwrap();
                        if result.page_info().has_next_page {
                            self.next_page = result.page_info().end_cursor.clone().into();
                        } else {
                            self.next_page = Maybe::None;
                        }
                        self.total_count = Some(result.total_count());
                        let mut iter = result.into_iter().skip(pos.index as usize);

                        match iter.next() {
                            None => None,
                            Some(item) => {
                                loop {
                                    match iter.next() {
                                        Some(v) => self.queued.push_back(v),
                                        None => break,
                                    }
                                }
                                pos.index += 1;
                                self.page_handler.set_page_position(&pos);
                                Some(item)
                            }
                        }
                    }
                }
            }
        }
    }
}
