use std::{
	cmp::Ordering,
	env,
	fmt,
	fs,
	io,
	path::PathBuf,
};
use cursive::{
	Cursive,
	traits::*,
	theme::*,
	utils::{
		markup::*,
	},
	event::{
		Event,
		Key,
	},
	view::{
		Nameable,
	},
	views::{
		Dialog,
		TextView,
		EditView,
		OnEventView,
		TextArea,
		LinearLayout,
	},
};
use cursive_tree_view::{
	Placement,
	TreeView,
};
use cursive_tabs::{
	Align, 
	TabPanel}
;


#[derive(Debug)]
struct TreeEntry {
	name: String,
	dir: Option<PathBuf>,
}

impl fmt::Display for TreeEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)
	}
}


fn collect_entries(dir: &PathBuf, entries: &mut Vec<TreeEntry>) -> io::Result<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();

			if path.is_dir() {
				entries.push(TreeEntry {
					name: entry
						.file_name()
						.into_string()
						.unwrap_or_else(|_| "".to_string()),
					dir: Some(path),
				});
			} else if path.is_file() {
				entries.push(TreeEntry {
					name: entry
						.file_name()
						.into_string()
						.unwrap_or_else(|_| "".to_string()),
					dir: None,
				});
			}
		}
	}
	Ok(())
}

fn expand_tree(tree: &mut TreeView<TreeEntry>, parent_row: usize, dir: &PathBuf) {
	let mut entries = Vec::new();
	collect_entries(dir, &mut entries).ok();

	entries.sort_by(|a, b| match (a.dir.is_some(), b.dir.is_some()) {
		(true, true) | (false, false) => a.name.cmp(&b.name),
		(true, false) => Ordering::Less,
		(false, true) => Ordering::Greater,
	});

	for i in entries {
		if i.dir.is_some() {
			tree.insert_container_item(i, Placement::LastChild, parent_row);
		} else {
			tree.insert_item(i, Placement::LastChild, parent_row);
		}
	}
}


fn main() {
	let mut file_tree= TreeView::<TreeEntry>::new();
	let path = env::current_dir().expect("No working directory");
	
	file_tree.insert_item(
		TreeEntry {
			name: path.file_name().unwrap().to_str().unwrap().to_string(),
			dir: Some(path.clone()),
		},
		Placement::After,
		0,
	);
	
	expand_tree(&mut file_tree, 0, &path);

	file_tree.set_on_collapse(|siv: &mut Cursive, row, is_collapsed, children| {
		if !is_collapsed && children == 0 {
			siv.call_on_name("tree", move |file_tree: &mut TreeView<TreeEntry>| {
				if let Some(dir) = file_tree.borrow_item(row).unwrap().dir.clone() {
					expand_tree(file_tree, row, &dir);
				}
			});
		}
	});
	
	let mut siv = cursive::default();
	// siv.add_layer(Dialog::around(file_tree.with_name("tree").scrollable()).title("File View"));
	siv.add_layer(
		Dialog::around(
			LinearLayout::horizontal()
				.child(TabPanel::new()
					.with_tab(TextView::new("Placeholder:\nGlobal Search").fixed_width(30).with_name(""))
					.with_tab_at(file_tree.with_name("tree").scrollable().fixed_width(30).with_name("פּ"), 0)
					.with_bar_alignment(Align::Start)
				)
				.child(TextArea::new().with_name("Title").scrollable().full_width().full_height())
				.full_width()
				.full_height(),
		).button("Quit", |s| s.quit())
	);
	
	siv.run();
	
}
