#![deny(warnings)]

extern crate git2;
extern crate regex;
extern crate semver;
extern crate structopt;

use git2::Repository;
use regex::Regex;
use semver::Version;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    app: String,
    #[structopt(help = "Set Semver Increment Type", default_value = "patch")]
    inc_type: String,
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

    println!("next version should be: {}", max_version.to_string())
}
