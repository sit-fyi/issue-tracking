use std::fmt::Display;
use askama::Template;
use graphql::{Query as GraphQLQuery, Pageable, HasPageInfo, HasTotalCount, PageInfo, Paged};
use author::Author;
use actor::Actor;

#[derive(Clone, Copy)]
pub enum Kind {
    Issue,
    PullRequest,
}


#[derive(Template)]
#[template(path = "timeline.graphql")]
pub struct Query<S> where S : AsRef<str> + Display {
    pub owner: S,
    pub repository: S,
    pub kind: &'static str,
    pub after: Option<String>,
    pub number: usize,
}

impl<S> Query<S> where S : AsRef<str> + Display {
    pub fn new(owner: S, repository: S, kind: Kind, number: usize) -> Self {
        let kind = match kind {
            Kind::Issue => "issue",
            Kind::PullRequest => "pullRequest",
        };
        Query { owner, repository, after: None, kind, number }
    }

}

impl<S> GraphQLQuery for Query<S> where S : AsRef<str> + Display + Copy {
    type Result = Response;
    fn query(&self) -> String {
        self.render().unwrap()
    }
}

impl<S> Pageable for Query<S> where S : AsRef<str> + Display + Copy {
    type Item = TimelineItem;
    fn after(&self, cursor: String) -> Self {
        Query { owner: self.owner, repository: self.repository, after: Some(cursor),
                kind: self.kind, number: self.number }
    }
}
#[derive(Deserialize, Debug)]
#[serde(tag = "__typename")]
pub enum Closer {
    #[serde(rename_all = "camelCase")]
    Commit {
        oid: String,
    },
    #[serde(rename_all = "camelCase")]
    PullRequest {
        number: usize,
    }
}

impl<'a> Into<String> for &'a Closer {
    fn into(self) -> String {
        match self {
            &Closer::Commit { ref oid } => format!("Closed with {}", oid),
            &Closer::PullRequest { number } => format!("Closed with pull request {}", number),
        }
    }
}

use files::Files;
use std::io::{Read, Cursor};
impl<'a> Into<Files<&'a str, Box<Read>>> for &'a Closer {

    fn into(self) -> Files<&'a str, Box<Read>> {
        let s: String = self.into();
        Files(vec![(".type/Commented", Box::new(&b""[..])),
                   ("text", Box::new(Cursor::new(s.into_bytes())))])
    }

}


#[derive(Deserialize, Debug)]
#[serde(tag = "__typename")]
pub enum TimelineItem {
    Commit {},
    #[serde(rename_all = "camelCase")]
    IssueComment {
        url: String,
        body: String,
        created_at: String,
        updated_at: String,
        author: Option<Author>,
    },
    CrossReferencedEvent {},
    #[serde(rename_all = "camelCase")]
    ClosedEvent {
        actor: Option<Actor>,
        closer: Option<Closer>,
        created_at: String,
    },
    #[serde(rename_all = "camelCase")]
    MergedEvent {
        actor: Option<Actor>,
        created_at: String,
    },
    #[serde(rename_all = "camelCase")]
    ReopenedEvent {
        actor: Option<Actor>,
        created_at: String,
    },
    SubscribedEvent {},
    UnsubscribedEvent {},
    ReferencedEvent {},
    AssignedEvent {},
    UnassignedEvent {},
    LabeledEvent {},
    UnlabeledEvent {},
    MilestonedEvent {},
    DemilestonedEvent {},
    RenamedTitleEvent {},
    #[serde(rename_all = "camelCase")]
    LockedEvent {
        actor: Option<Actor>,
        created_at: String,
    },
    #[serde(rename_all = "camelCase")]
    UnlockedEvent {
        actor: Option<Actor>,
        created_at: String,
    },
    HeadRefDeletedEvent {},
    HeadRefRestoredEvent {},
    HeadRefForcePushedEvent {},
    BaseRefForcePushedEvent {},
    ReviewRequestedEvent {},
    ReviewRequestedRemovedEvent {},
    ReviewDismissedEvent {},
    DeployedEvent {},
    CommitCommentThread {},
    PullRequestReview {},
    PullRequestReviewThread {},
    PullRequestReviewComment {},

}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimelineNode {
    timeline: Paged<TimelineItem>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IssueNode {
    issue: TimelineNode,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    repository: IssueNode,
}

impl HasPageInfo for Response {
    fn page_info(&self) -> &PageInfo {
        &self.repository.issue.timeline.page_info
    }
}

impl HasTotalCount for Response {
    fn total_count(&self) -> usize {
        self.repository.issue.timeline.total_count
    }
}

impl IntoIterator for Response {
    type Item = TimelineItem;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.repository.issue.timeline.into_iter()
    }
}

