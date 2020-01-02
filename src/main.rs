#![deny(warnings)]

extern crate git2;

use git2::Repository;

fn main() {
    let repo = match Repository::open("./") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let tags = match repo.tag_names(Some("www")) {
        Ok(tags) => tags,
        Err(e) => panic!("could not find any tags {}", e),
    };

    println!("found {} tags", tags.len())
}
