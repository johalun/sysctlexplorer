// tree_entry.rs

use std::cmp;
use std::fmt;
use sysctl::*;

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub name: String,
    pub ctlname: String,
    pub depth: usize,
    pub ctl: Option<Ctl>,
}

impl TreeEntry {
    pub fn new(ctlname: &str, depth: usize, ctl: Option<Ctl>) -> TreeEntry {
        TreeEntry {
            name: ctlname
                .split(".")
                .nth(depth)
                .expect("name split")
                .to_owned(),
            ctlname: ctlname.to_owned(),
            depth: depth,
            ctl: ctl,
        }
    }
    pub fn path(&self) -> String {
        let parts: Vec<&str> = self.ctlname.split(".").collect();
        let path = parts[0..self.depth + 1].join(".");
        path
    }
}

impl fmt::Display for TreeEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl cmp::Eq for TreeEntry {}

impl cmp::PartialEq for TreeEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl cmp::Ord for TreeEntry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl cmp::PartialOrd for TreeEntry {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
