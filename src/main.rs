#![deny(warnings)]

extern crate git2;
extern crate regex;
extern crate semver;
extern crate serde;
extern crate structopt;
extern crate toml;

use git2::{DiffOptions, Repository};
use regex::Regex;
use semver::Version;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use structopt::StructOpt;

#[derive(Deserialize)]
struct Config {
    access_token: String,
    repo_path: String,
}

#[derive(StructOpt)]
struct Cli {
    app: String,
    #[structopt(help = "Set Semver Increment Type", default_value = "patch")]
    inc_type: String,
    #[structopt(help = "Previous version", short = "p", long = "previous")]
    previous_version: Option<String>,
}

fn get_pathspecs(app: &str) -> Vec<&'static str> {
    let paths = match app.as_ref() {
        "admin" => vec!["admin/"],
        "api" => vec!["server/"],
        "ios" => vec!["client/ios", "client/app"],
        "android" => vec!["client/android", "client/app"],
        "www" => vec!["www/"],
        _ => vec![],
    };
    return paths;
}

fn main() {
    let args = Cli::from_args();

    let config_file = Path::new("./config.toml");
    let conf: String = fs::read_to_string(config_file).unwrap();
    let config: Config = toml::from_str(&conf).unwrap();

    if false {
        println!("config token: {}", config.access_token);
    }

    // let url = "git@github.com:How-2-Do/doer.git";
    // let repo = match Repository::clone(url, "/tmp/doer-release-notes") {
    //     Ok(repo) => repo,
    //     Err(e) => panic!("failed to clone: {}", e),
    // };

    let repo = match Repository::open(config.repo_path) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let tag_pattern = format!("{}/**", args.app);
    let tags = match repo.tag_names(Some(tag_pattern.as_ref())) {
        Ok(tags) => tags,
        Err(e) => panic!("could not find any tags {}", e),
    };

    let mut max_version = Version::parse("0.0.0").unwrap();

    for tag in tags.into_iter() {
        if let Some(name) = &tag {
            let re = Regex::new(r"(.*)/(.*)").unwrap();
            let matches = match re.captures(name) {
                Some(matches) => matches,
                None => panic!("No prior tags matching expected format."),
            };
            if matches[1] == args.app {
                let version = match Version::parse(&matches[2]) {
                    Ok(version) => version,
                    Err(_) => Version::parse("0.0.0").unwrap(),
                };
                if version > max_version {
                    max_version = version;
                }
            }
        }
    }

    let most_recent_version_string = match args.previous_version {
        Some(version) => format!("refs/tags/{}/{}", args.app, version),
        None => format!("refs/tags/{}/{}", args.app, max_version.to_string()),
    };

    match args.inc_type.as_ref() {
        "major" => max_version.increment_major(),
        "minor" => max_version.increment_minor(),
        "patch" => max_version.increment_patch(),
        _ => max_version.increment_patch(),
    };

    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL);

    let references = repo.references().unwrap();
    for reference in references {
        let reference = reference.unwrap();
        let name = reference.name().unwrap_or("Unknown");
        if name == most_recent_version_string {
            let target = match reference.target() {
                Some(target) => target,
                None => panic!("Error getting target"),
            };

            let _hide = match revwalk.hide(target) {
                Ok(hide) => hide,
                Err(e) => panic!("why are we panicking... {}", e),
            };
        }
    }

    let mut features: Vec<String> = vec![];
    let mut bugs: Vec<String> = vec![];
    let mut chores: Vec<String> = vec![];
    let mut others: Vec<String> = vec![];

    for commit_id in revwalk.into_iter() {
        let commit_id = match commit_id {
            Ok(id) => id,
            Err(e) => panic!("Error with getting commit in revwalk: {}", e),
        };
        let find_commit = repo.find_commit(commit_id);
        let commit = match find_commit {
            Ok(c) => c,
            Err(e) => panic!("Error getting commit with id {}: {}", commit_id, e),
        };
        for parent in commit.parents() {
            let mut diffopts = DiffOptions::new();
            for path in get_pathspecs(args.app.as_ref()).into_iter() {
                diffopts.pathspec(path);
            }
            let diff = repo
                .diff_tree_to_tree(
                    Some(&parent.tree().unwrap()),
                    Some(&commit.tree().unwrap()),
                    Some(&mut diffopts),
                )
                .unwrap();
            if diff.stats().unwrap().files_changed() > 0 {
                let commit_message_bytes = commit.message_bytes();
                let message_string = String::from_utf8_lossy(commit_message_bytes);
                let message = message_string.lines().next().unwrap();
                if &message[0..9] == "[feature]" {
                    features.push(message[9..].to_string());
                } else if &message[0..5] == "[bug]" || &message[0..5] == "[fix]" {
                    bugs.push(message[5..].to_string());
                } else if &message[0..7] == "[chore]" {
                    chores.push(message[7..].to_string());
                } else {
                    others.push(message.to_string());
                }
            }
        }
    }

    if features.len() > 0 {
        println!("### 🚀 Features");
        for feature in features {
            println!("* {}", feature);
        }
    }

    if bugs.len() > 0 {
        println!("### 🐛 Fixes");
        for bug in bugs {
            println!("* {}", bug);
        }
    }

    if chores.len() > 0 {
        println!("### 📝 Chores");
        for chore in chores {
            println!("* {}", chore);
        }
    }

    if others.len() > 0 {
        println!("### Everything else");
        for other in others {
            println!("* {}", other);
        }
    }

    println!("next version should be: {}", max_version.to_string())
}
