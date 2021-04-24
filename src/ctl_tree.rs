// ctl_tree.rs

use crate::tree_entry::*;
use std::sync::Mutex;
use sysctl::*;

pub struct CtlTree {
    ctls: Vec<Ctl>,
    filter: Mutex<Option<String>>,
}

impl CtlTree {
    pub fn new() -> Self {
        let ctls: Vec<Ctl> = sysctl::CtlIter::root().filter_map(Result::ok).collect();
        CtlTree {
            ctls: ctls,
            filter: Mutex::new(None),
        }
    }
    pub fn contents(&self, path: &str) -> Vec<TreeEntry> {
        let mut v: Vec<TreeEntry> = vec![];
        let filter_guard = self.filter.lock().unwrap();
        let filter = filter_guard.as_ref();
        if path == "" {
            for ctl in &self.ctls {
                let flags = match ctl.flags() {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                if !flags.contains(sysctl::CtlFlags::SKIP) {
                    if let Ok(ctlname) = ctl.name() {
                        if filter.is_some() {
                            if !ctlname.contains(filter.unwrap()) {
                                continue;
                            }
                        }
                        let e = TreeEntry::new(&ctlname, 0, None);
                        if !v.contains(&e) {
                            v.push(e);
                        }
                    } else {
                        warn!("Could not get name for {:?}. Skipping.", ctl);
                    }
                }
            }
        } else {
            for ctl in &self.ctls {
                let flags = match ctl.flags() {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                if !flags.contains(sysctl::CtlFlags::SKIP) {
                    if let Ok(ctlname) = ctl.name() {
                        if !ctlname.starts_with(path) {
                            continue;
                        }
                        if filter.is_some() {
                            if !ctlname.contains(filter.unwrap()) {
                                continue;
                            }
                        }
                        let depth: usize = path.matches(".").count();
                        let parts: Vec<&str> = ctlname.split(".").collect();
                        let ctlpath = parts[0..parts.len() - 1].join(".");
                        if path == ctlpath {
                            let e = TreeEntry::new(&ctlname, depth + 1, Some(ctl.clone()));
                            if !v.contains(&e) {
                                v.push(e);
                            }
                        } else if ctlname.starts_with(&format!("{}.", path)) {
                            let e = TreeEntry::new(&ctlname, depth + 1, None);
                            if !v.contains(&e) {
                                v.push(e);
                            }
                        }
                    } else {
                        warn!("Could not get name for {:?}. Skipping.", ctl);
                    }
                }
            }
        }
        v.sort();
        v
    }

    pub fn filter(&self, string: Option<String>) {
        *self.filter.lock().unwrap() = string;
    }
}
