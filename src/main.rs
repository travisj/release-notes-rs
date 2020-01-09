#![deny(warnings)]

extern crate git2;
extern crate regex;
extern crate semver;
extern crate structopt;

use git2::{DiffOptions, Repository};

use regex::Regex;
use semver::Version;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    app: String,
    #[structopt(help = "Set Semver Increment Type", default_value = "patch")]
    inc_type: String,
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

    let repo = match Repository::open("/Users/travisj/Projects/doer") {
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

    let most_recent_version_string = format!("refs/tags/{}/{}", args.app, max_version.to_string());

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

    for commit_id in revwalk.into_iter() {
        let commit = match commit_id {
            Ok(id) => match repo.find_commit(id) {
                Ok(commit) => commit,
                Err(e) => panic!("Error getting commit with id {}: {}", id, e),
            },
            Err(e) => panic!("Error with getting commit in revwalk: {}", e),
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
                println!(
                    "* {}",
                    String::from_utf8_lossy(commit.message_bytes())
                        .lines()
                        .next()
                        .unwrap()
                );
            }
        }
    }

    println!("next version should be: {}", max_version.to_string())
}
