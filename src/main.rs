// main.rs

extern crate cursive;
extern crate cursive_tree_view;
extern crate sysctl;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simplelog;

use std::fs;

use ctl_tree::*;
use cursive::traits::*;
use cursive::views::*;
use cursive::Cursive;
use cursive_tree_view::{Placement, TreeView};
use simplelog::*;
use sysctl::*;
use tree_entry::*;

pub mod ctl_tree;
pub mod tree_entry;

lazy_static! {
    static ref CTLTREE: CtlTree = CtlTree::new();
}

static USAGE: &str = "Instructions
------------
Up/Down:    Navigate tree or scroll textview.
Tab:        Toggle focus between tree and details.
Enter:      Expand/collapse tree or view selected sysctl.
'e':        Edit value in selected sysctl.
's':        Search sysctl by name (coming soon).
'q':        Quit program.
";

fn main() {
    CombinedLogger::init(vec![
        // TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            fs::File::create("/tmp/sysctlexplorer.log").expect("Create log file"),
        ),
    ])
    .expect("Init combined logger");

    fn expand_tree(tree: &mut TreeView<TreeEntry>, parent_row: usize, path: &str) {
        let ctls: Vec<TreeEntry> = CTLTREE.contents(path);
        for e in ctls {
            if e.ctl.is_some() {
                tree.insert_item(e.clone(), Placement::LastChild, parent_row)
                    .expect("insert item");
            } else {
                tree.insert_container_item(e.clone(), Placement::LastChild, parent_row)
                    .expect("insert container");
            }
        }
    }

    let mut tree = TreeView::<TreeEntry>::new();

    // Scope this so temporary variables gets released
    {
        let root: Vec<TreeEntry> = CTLTREE.contents("");

        let mut i = 0;
        for e in root {
            i = tree
                .insert_container_item(e.clone(), Placement::After, i)
                .expect("insert container");
        }
    }

    tree.set_on_collapse(|siv: &mut Cursive, row, is_collapsed, children| {
        if !is_collapsed && children == 0 {
            // Expand entry
            siv.call_on_id("tree", move |tree: &mut TreeView<TreeEntry>| {
                let path: String = tree.borrow_item(row).expect("borrow item from tree").path();
                expand_tree(tree, row, &path);
            });
        }
    });

    tree.set_on_submit(|siv: &mut Cursive, row| {
        let e: TreeEntry = siv
            .call_on_id("tree", move |tree: &mut TreeView<TreeEntry>| {
                tree.borrow_item(row)
                    .expect("borrow item from tree")
                    .clone()
            })
            .expect("call on id");
        siv.call_on_id("text", move |text: &mut TextView| {
            if let Some(ctl) = e.ctl {
                if let (Ok(n), Ok(v), Ok(vt), Ok(d)) =
                    (ctl.name(), ctl.value(), ctl.value_type(), ctl.description())
                {
                    let mut s = format!("Name: {}\n", n);
                    s.push_str(&format!("Description: {}\n", d));
                    s.push_str(&format!("Value ({:?}): {}\n", vt, v));
                    text.set_content(s);
                } else {
                    warn!("Could not get information for {:?}", ctl);
                    text.set_content(format!("Could not get value for: {:?}", ctl));
                }
            }
        });
    });

    fn edit(siv: &mut Cursive) {
        if let Some(row) = siv
            .call_on_id("tree", move |tree: &mut TreeView<TreeEntry>| tree.row())
            .expect("call_on_id")
        {
            let e: TreeEntry = siv
                .call_on_id("tree", move |tree: &mut TreeView<TreeEntry>| {
                    tree.borrow_item(row)
                        .expect("borrow item from tree")
                        .clone()
                })
                .expect("call on id");
            if let Some(ctl) = e.ctl {
                if let (Ok(old_value), Ok(value_type)) = (ctl.value_string(), ctl.value_type()) {
                    // Update textview
                    let ctl2 = ctl.clone();
                    siv.call_on_id("text", move |text: &mut TextView| {
                        if let (Ok(n), Ok(v), Ok(d)) =
                            (ctl2.name(), ctl2.value(), ctl2.description())
                        {
                            let mut s = format!("Name: {}\n", n);
                            s.push_str(&format!("Description: {}\n", d));
                            s.push_str(&format!("Value ({:?}): {}\n", value_type, v));
                            text.set_content(s);
                        } else {
                            warn!("Could not get information for {:?}", ctl2);
                            text.set_content(format!("Could not get value for: {:?}", ctl2));
                        }
                    });
                    if let Ok(flags) = ctl.flags() {
                        if flags.bits() & crate::CTLFLAG_WR == 1 {
                            siv.add_layer(
                                Dialog::around(TextView::new(format!("This sysctl is read only.")))
                                    .button("Close", |siv: &mut Cursive| {
                                        siv.pop_layer();
                                    }),
                            );
                            return;
                        }
                    }
                    // Show edit dialog
                    siv.add_layer(
                        Dialog::new()
                            .title("Enter new value")
                            // Padding is (left, right, top, bottom)
                            .padding((1, 1, 1, 0))
                            .content(
                                EditView::new()
                                    .content(old_value)
                                    .on_submit(move |siv: &mut Cursive, s: &str| {
                                        // if ctl.set_value_string(s).is_ok() {
                                        let _e: Result<String, SysctlError> = ctl.set_value_string(s)
                                            .and_then(|s: String| {
                                                siv.pop_layer();
                                                // Update textview
                                                let ctl2 = ctl.clone();
                                                siv.call_on_id(
                                                    "text",
                                                    move |text: &mut TextView| {
                                                        if let (Ok(n), Ok(v), Ok(d)) = (
                                                            ctl2.name(),
                                                            ctl2.value(),
                                                            ctl2.description(),
                                                        ) {
                                                            let mut s = format!("Name: {}\n", n);
                                                            s.push_str(&format!(
                                                                "Description: {}\n",
                                                                d
                                                            ));
                                                            s.push_str(&format!(
                                                                "Value ({:?}): {}\n",
                                                                value_type, v
                                                            ));
                                                            text.set_content(s);
                                                        } else {
                                                            warn!(
                                                                "Could not get information for {:?}",
                                                                ctl2
                                                            );
                                                            text.set_content(format!(
                                                                "Could not get value for: {:?}",
                                                                ctl2
                                                            ));
                                                        }
                                                    },
                                                );
                                                Ok(s)
                                            })
                                            .or_else(|e: SysctlError| {
                                                siv.add_layer(
                                                    Dialog::around(TextView::new(format!(
                                                        "Error: {:?}",
                                                        e
                                                    )))
                                                    .button("Close", |siv: &mut Cursive| {
                                                        siv.pop_layer();
                                                    }),
                                                );
                                                Ok("dummy".to_owned())
                                            });
                                        info!("EditView submit");
                                    })
                                    .with_id("edit"),
                            )
                            .button("Cancel", |siv| {
                                siv.pop_layer();
                            }),
                    );
                }
            } else {
                siv.add_layer(
                    Dialog::around(TextView::new("Can not edit this entry")).button(
                        "Close",
                        |siv: &mut Cursive| {
                            siv.pop_layer();
                        },
                    ),
                );
            }
        }
    }

    // Setup Cursive
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback('e', |s| edit(s));
    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(tree.with_id("tree").min_width(38))
                .child(TextView::new(USAGE).with_id("text").scrollable()),
        )
        .title("The sysctl explorer")
        .full_screen(),
    );

    siv.run();
}
