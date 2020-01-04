#![deny(warnings)]

extern crate git2;
extern crate regex;
extern crate semver;
extern crate structopt;

use git2::{Commit, Repository};

use regex::Regex;
use semver::Version;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    app: String,
    #[structopt(help = "Set Semver Increment Type", default_value = "patch")]
    inc_type: String,
}

fn print_commit(commit: &Commit) {
    println!("commit {}", commit.id());

    if commit.parents().len() > 1 {
        print!("Merge:");
        for id in commit.parent_ids() {
            print!(" {:.8}", id);
        }
        println!();
    }

    let author = commit.author();
    println!("Author: {}", author);
    // print_time(&author.when(), "Date:   ");
    println!();

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        println!("    {}", line);
    }
    println!();
}

// fn print_time(time: &Time, prefix: &str) {
//     let (offset, sign) = match time.offset_minutes() {
//         n if n < 0 => (-n, '-'),
//         n => (n, '+'),
//     };
//     let (hours, minutes) = (offset / 60, offset % 60);
//     let ts = time::Timespec::new(time.seconds() + (time.offset_minutes() as i64) * 60, 0);
//     let time = time::at(ts);

//     println!(
//         "{}{} {}{:02}{:02}",
//         prefix,
//         time.strftime("%a %b %e %T %Y").unwrap(),
//         sign,
//         hours,
//         minutes
//     );
// }

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

    println!("found {} tags", tags.len());

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

    match args.inc_type.as_ref() {
        "major" => max_version.increment_major(),
        "minor" => max_version.increment_minor(),
        "patch" => max_version.increment_patch(),
        _ => max_version.increment_patch(),
    };

    let mut revwalk = repo.revwalk().unwrap();
    match revwalk.push_head() {
        Ok(_) => println!("push_head success"),
        Err(e) => println!("push_head failure: {}", e),
    };
    revwalk.set_sorting(git2::Sort::TOPOLOGICAL);

    for commit_id in revwalk.into_iter() {
        println!("here:");
        let commit = match commit_id {
            Ok(id) => {
                println!("Commit ID: {}", id);
                match repo.find_commit(id) {
                    Ok(commit) => commit,
                    Err(e) => panic!("Error getting commit with id {}: {}", id, e),
                }
            }
            Err(e) => panic!("Error with getting commit in revwalk: {}", e),
        };
        print_commit(&commit);
    }

    println!("next version should be: {}", max_version.to_string())
}
