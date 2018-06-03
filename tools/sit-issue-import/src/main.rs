extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

extern crate reqwest;

#[macro_use] extern crate clap;
extern crate console;

#[macro_use] extern crate askama;

extern crate xdg;
extern crate config;

extern crate pbr;

extern crate regex;

extern crate git2;

extern crate sit_core;

mod graphql;

mod issues;
mod pull_requests;
mod timeline;
mod author;
mod actor;
mod files;
use files::Files;

use std::env;
use std::path::PathBuf;
use clap::{App, Arg};

const GITHUB_GRAPHQL : &str = "https://api.github.com/graphql";

fn main() {
    ::std::process::exit(real_main())
}

#[derive(Deserialize)]
struct GitHubProviderConfig {
    token: String,
}

#[derive(Deserialize)]
struct Config {
    github: Option<GitHubProviderConfig>,
}

fn real_main() -> i32 {

    let cwd = env::current_dir().expect("can't get currenGt working directory");

    let matches = App::new("SIT Import")
        .version(crate_version!())
        .about("Imports foreign issues into SIT repositories")
        .global_settings(&[clap::AppSettings::ColoredHelp, clap::AppSettings::ColorAuto])
        .arg(Arg::with_name("working_directory")
            .short("d")
            .help("Working directory"))
        .arg(Arg::with_name("repository")
            .short("r")
            .long("repository")
            .takes_value(true)
            .help("Point to a specific directory of SIT's repository"))
        .arg(Arg::with_name("config")
                 .short("c")
                 .long("config")
                 .takes_value(true)
                 .help("Configuration file"))
        .arg(Arg::with_name("SOURCE")
                 .required(true)
                 .long_help("Where to import from\
                 \nFor GitHub: https://github.com/OWNER/REPO"))
        .get_matches();

    let working_dir = matches.value_of("working_directory").map(PathBuf::from).unwrap_or(cwd);

    let repo_path = matches.value_of("repository").map(PathBuf::from)
        .or_else(|| sit_core::Repository::find_in_or_above(".sit",&working_dir))
        .expect("Can't find a repository");
    let repo = sit_core::Repository::open(&repo_path)
        .expect("can't open repository");

    let source = matches.value_of("SOURCE").unwrap();

    if source.starts_with("https://github.com/") {

        git2::Repository::clone("https://github.com/sit-it/issue-tracking.git", repo.modules_path().join("issue-tracking")).unwrap();

        if !matches.is_present("config") {
            eprintln!("-c/--config required for GitHub to configure the token");
            return 1;
        }
        let mut cfg = config::Config::new();
        cfg.merge(config::File::with_name(matches.value_of("config").unwrap()).required(true)).unwrap();
        let configuration: Config = cfg.try_into().expect("Can't deserialize configuration");

        if configuration.github.is_none() {
            eprintln!("github provider configuration required");
            return -1;
        }

        let re = regex::Regex::new("https://github.com/([^/]+)/([^/]+)(\\.git)?").unwrap();
        let captures = re.captures(source).unwrap();
        if captures.len() < 3 {
            println!("Invalid source URL");
            return 1;
        }
        let owner = &captures[1];
        let repository = &captures[2];

        let mut req0 = graphql::PageableRequest::new(GITHUB_GRAPHQL, &configuration.github.as_ref().unwrap().token, issues::Query::new(owner, repository), graphql::MemoryPageHandler::default());
        let req = graphql::PageableRequest::new(GITHUB_GRAPHQL, &configuration.github.as_ref().unwrap().token, issues::Query::new(owner, repository), graphql::MemoryPageHandler::default());

        let mut preq0 = graphql::PageableRequest::new(GITHUB_GRAPHQL, &configuration.github.as_ref().unwrap().token, pull_requests::Query::new(owner, repository), graphql::MemoryPageHandler::default());
        let preq = graphql::PageableRequest::new(GITHUB_GRAPHQL, &configuration.github.as_ref().unwrap().token, pull_requests::Query::new(owner, repository), graphql::MemoryPageHandler::default());

        use sit_core::Item;

        if req0.next().is_none() {
            // Nothing to see
            return 0
        }
        let prs = match preq0.next() {
            None => 0 as u64,
            Some(_) => preq0.total_count().unwrap() as u64,
        };

        let mut progress_bar = pbr::ProgressBar::new(req0.total_count().unwrap() as u64 + prs);

        progress_bar.message("[ Importing issues ] ");
        progress_bar.set(0);

        for ext_issue in req {
            let issue = repo.new_named_item(format!("github-issue-{}", ext_issue.number)).unwrap();

            progress_bar.message(&format!("[ #{} {} ] ", ext_issue.number, ext_issue.title));

            issue.new_record(Files::from(vec![(".type/SummaryChanged", &b""[..]),
                                              (".timestamp", ext_issue.created_at.as_bytes()),
                                              ("text", ext_issue.title.as_bytes()),
                                              (".imported", ext_issue.url.as_bytes())])
                                 .followed_by(ext_issue.author.as_ref().into())
                                 .into_iter(), true).unwrap();

            issue.new_record(Files::from(vec![(".type/DetailsChanged", &b""[..]),
                                              (".timestamp", ext_issue.created_at.as_bytes()),
                                              ("text", ext_issue.body.as_bytes()),
                                              (".imported", ext_issue.url.as_bytes())])
                                 .followed_by(ext_issue.author.as_ref().into())
                                 .into_iter(), true).unwrap();

            let treq = graphql::PageableRequest::new(GITHUB_GRAPHQL, &configuration.github.as_ref().unwrap().token,
                                                     timeline::Query::new(owner, repository, timeline::Kind::Issue, ext_issue.number), graphql::MemoryPageHandler::default());

            for item in treq {

                match item {
                    timeline::TimelineItem::IssueComment { created_at, body, author, url, .. } => {
                        issue.new_record(Files::from(vec![(".type/Commented", &b""[..]),
                                              (".timestamp", created_at.as_bytes()),
                                              ("text", body.as_bytes()),
                                              (".imported", url.as_bytes())])
                                             .followed_by(author.as_ref().into())
                                             .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::ClosedEvent { created_at, actor, closer } => {
                         issue.new_record(Files::from(vec![(".type/Closed", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", ext_issue.url.as_bytes())])
                                              .followed_by(closer.as_ref().into())
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::ReopenedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Reopened", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", ext_issue.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::LockedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Locked", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", ext_issue.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::UnlockedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Unlocked", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", ext_issue.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();

                    },
                    _ => (),
                }
            }

            progress_bar.inc();
        }

       let client = reqwest::Client::builder().build().unwrap();

        progress_bar.message("[ Importing pull requests ] ");

        for pr in preq {

            let issue = repo.new_named_item(format!("github-pr-{}", pr.number)).unwrap();
            progress_bar.message(&format!("[ #{} {} ] ", pr.number, pr.title));

            issue.new_record(Files::from(vec![(".type/SummaryChanged", &b""[..]),
                                  (".timestamp", pr.created_at.as_bytes()),
                                  ("text", pr.title.as_bytes()),
                                  (".imported", pr.url.as_bytes())])
                                 .followed_by(pr.author.as_ref().into())
                                 .into_iter(), true).unwrap();

            let mut response = client.get(&format!("{}.patch", pr.url)).send().unwrap();
            let patch = response.text().unwrap();

            let mut mr_rec = vec![(".type/DetailsChanged", &b""[..]),
                                  (".type/MergeRequested", &b""[..]),
                                  (".timestamp", pr.created_at.as_bytes()),
                                  ("text", pr.body.as_bytes()),
                                  (".imported", pr.url.as_bytes())];

            if response.status().is_success() {
                mr_rec.push(("git/pr.patch", patch.as_bytes()));
            }

            issue.new_record(Files::from(mr_rec).followed_by(pr.author.as_ref().into()).into_iter(), true).unwrap();

            let treq = graphql::PageableRequest::new(GITHUB_GRAPHQL, &configuration.github.as_ref().unwrap().token,
                                                     timeline::Query::new(owner, repository, timeline::Kind::PullRequest, pr.number), graphql::MemoryPageHandler::default());

            for item in treq {
                match item {
                    timeline::TimelineItem::IssueComment { created_at, body, author, url, .. } => {
                        issue.new_record(Files::from(vec![(".type/Commented", &b""[..]),
                                              (".timestamp", created_at.as_bytes()),
                                              ("text", body.as_bytes()),
                                              (".imported", url.as_bytes())])
                                             .followed_by(author.as_ref().into())
                                             .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::ClosedEvent { created_at, actor, closer } => {
                         issue.new_record(Files::from(vec![(".type/Closed", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", pr.url.as_bytes())])
                                              .followed_by(closer.as_ref().into())
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::ReopenedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Reopened", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", pr.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::LockedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Locked", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", pr.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    timeline::TimelineItem::UnlockedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Unlocked", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", pr.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();

                    },
                    timeline::TimelineItem::MergedEvent { created_at, actor } => {
                         issue.new_record(Files::from(vec![(".type/Closed", &b""[..]),
                                               (".type/Merged", &b""[..]),
                                               (".timestamp", created_at.as_bytes()),
                                               (".imported", pr.url.as_bytes())])
                                              .followed_by(actor.as_ref().into())
                                              .into_iter(), true).unwrap();
                    },
                    _ => (),
                }
            }

            progress_bar.inc();
        }

        progress_bar.finish();


    } else {
        eprintln!("Unrecognized source provider: {}", source);
        return -1;
    }

    return 0;


}