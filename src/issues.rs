use std::fmt::Display;
use askama::Template;
use graphql::{Query as GraphQLQuery, Pageable, HasPageInfo, HasTotalCount, PageInfo, Paged};
use author::Author;

#[derive(Template)]
#[template(path = "issues.graphql")]
pub struct Query<S> where S : AsRef<str> + Display {
    pub owner: S,
    pub repository: S,
    pub after: Option<String>,
}

impl<S> Query<S> where S : AsRef<str> + Display {
    pub fn new(owner: S, repository: S) -> Self {
        Query { owner, repository, after: None }
    }

}

impl<S> GraphQLQuery for Query<S> where S : AsRef<str> + Display + Copy {
    type Result = Response;
    fn query(&self) -> String {
        self.render().unwrap()
    }
}

impl<S> Pageable for Query<S> where S : AsRef<str> + Display + Copy {
    type Item = Issue;
    fn after(&self, cursor: String) -> Self {
        Query{ owner: self.owner, repository: self.repository, after: Some(cursor) }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub number: usize,
    pub url: String,
    pub state: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub author: Option<Author>,
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IssueNode {
    issues: Paged<Issue>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    repository: IssueNode,
}

impl HasPageInfo for Response {
    fn page_info(&self) -> &PageInfo {
        &self.repository.issues.page_info
    }
}

impl HasTotalCount for Response {
    fn total_count(&self) -> usize {
        self.repository.issues.total_count
    }
}

impl IntoIterator for Response {
    type Item = Issue;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.repository.issues.into_iter()
    }
}
