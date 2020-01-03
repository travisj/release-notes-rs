#![deny(warnings)]

extern crate git2;
extern crate regex;
extern crate semver;

use git2::Repository;
use regex::Regex;
use semver::Version;

fn main() {
    let repo = match Repository::open("/Users/travisj/Projects/doer") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let tags = match repo.tag_names(Some("www/*")) {
        Ok(tags) => tags,
        Err(e) => panic!("could not find any tags {}", e),
    };

    println!("found {} tags", tags.len());

    let versions: Vec<semver::Version> = tags
        .into_iter()
        .map(|tag| {
            let name = match tag {
                Some(name) => name,
                None => "0.0.0",
            };
            let re = Regex::new(r"www/(.*)").unwrap();
            let matches = re.captures(name).unwrap();
            match Version::parse(&matches[1]) {
                Ok(version) => version,
                Err(e) => panic!("could not parse {}", e),
            }
        })
        .collect();

    let mut max_version = Version::parse("0.0.0").unwrap();
    for version in versions {
        if version > max_version {
            max_version = version
        }
    }
    max_version.increment_patch();
    println!("next version should be: {}", max_version.to_string())
}
