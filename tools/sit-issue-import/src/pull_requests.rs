use std::fmt::Display;
use askama::Template;
use graphql::{Query as GraphQLQuery, Pageable, HasPageInfo, HasTotalCount, PageInfo, Paged};
use author::Author;

#[derive(Template)]
#[template(path = "pull_requests.graphql")]
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
    type Item = PullRequest;
    fn after(&self, cursor: String) -> Self {
        Query{ owner: self.owner, repository: self.repository, after: Some(cursor) }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub number: usize,
    pub url: String,
    pub state: String,
    pub title: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub author: Option<Author>,
    pub merged: bool,
    pub merged_at: Option<String>,
}


#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestNode {
    pull_requests: Paged<PullRequest>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    repository: PullRequestNode,
}

impl HasPageInfo for Response {
    fn page_info(&self) -> &PageInfo {
        &self.repository.pull_requests.page_info
    }
}

impl HasTotalCount for Response {
    fn total_count(&self) -> usize {
        self.repository.pull_requests.total_count
    }
}

impl IntoIterator for Response {
    type Item = PullRequest;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.repository.pull_requests.into_iter()
    }
}
