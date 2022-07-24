use std::{
	cmp::Ordering,
	env,
	fmt,
	fs,
	io,
	str::FromStr,
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
		Margins,
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
//use cursive_markup::*;


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

			// There's probably a much faster way of ignoring dotfiles.
			// In fact, there's probably a much faster way of building this tree.
			// This works for the time being, leaving it in until it becomes problematic.
			if !path.file_name().unwrap().to_str().unwrap().starts_with(".") {
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

fn theme(siv: &Cursive) -> Theme {
	let mut theme = siv.current_theme().clone();
	
	/*theme.palette[PaletteColor::Background] = Color::TerminalDefault;
	theme.palette[PaletteColor::View] = Color::TerminalDefault;
	theme.palette[PaletteColor::Shadow] = Color::TerminalDefault;
	theme.palette[PaletteColor::TitlePrimary] = Color::TerminalDefault;
	theme.palette[PaletteColor::TitleSecondary] = Color::TerminalDefault;
	theme.palette[PaletteColor::Primary] = Color::TerminalDefault;
	theme.palette[PaletteColor::Secondary] = Color::TerminalDefault;
	theme.palette[PaletteColor::Tertiary] = Color::TerminalDefault;
	theme.palette[PaletteColor::Highlight] = Color::from_256colors(6);
	theme.palette[PaletteColor::HighlightInactive] = Color::from_256colors(4);
	theme.palette[PaletteColor::HighlightText] = Color::from_256colors(0);*/
	theme.shadow = false;
	theme.borders = BorderStyle::None;
	
	theme
}

fn main() {
	let args: Vec<_> = env::args().collect();
	
	let mut siv = cursive::default();

	let theme = theme(&siv);
	siv.set_theme(theme);
	
	
	let cwd: PathBuf = env::current_dir().expect("No working directory");
	let library_dir: PathBuf = PathBuf::from_str(args.get(1).unwrap_or(&cwd.to_str().unwrap().to_string())).unwrap_or(cwd);

	
	// File tree setup

	let mut file_tree= TreeView::<TreeEntry>::new();

	file_tree.insert_item(
		TreeEntry {
			name: library_dir.file_name().unwrap().to_str().unwrap().to_string(),
			dir: Some(library_dir.clone()),
		},
		Placement::After,
		0,
	);
	
	expand_tree(&mut file_tree, 0, &library_dir);

	file_tree.set_on_collapse(|siv: &mut Cursive, row, is_collapsed, children| {
		if !is_collapsed && children == 0 {
			siv.call_on_name("tree", move |file_tree: &mut TreeView<TreeEntry>| {
				if let Some(dir) = file_tree.borrow_item(row).unwrap().dir.clone() {
					expand_tree(file_tree, row, &dir);
				}
			});
		}
	});
	
	
	// Editor setup
	
	let mut editor = TextArea::new().with_name("editor");
	
	
	// Set up main Cursive layer
	
	let sidebar_width: usize = 25;
	siv.add_fullscreen_layer(
			LinearLayout::horizontal()
				.child(TabPanel::new()
					// Tabs are placed in the order declared below, unless `with_tab_at()` is used.
					// The last tab created gets focused, so the "main" tab will be declared last,
					// but at the first (0th) position in the tab series.
					.with_tab(
						TextView::new("Placeholder:\nGlobal Search").fixed_width(sidebar_width).with_name("")
					)
					.with_tab(
						TextView::new("Placeholder:\nStarred Files").fixed_width(sidebar_width).with_name("")
					)
					.with_tab_at(
						file_tree.with_name("tree").scrollable().fixed_width(sidebar_width).with_name("פּ"), 
						0
					)
					.with_bar_alignment(Align::Start)
				)
				.child(
					editor.scrollable().full_width().full_height()
				)
				.full_width()
				.full_height(),
	);
	
	siv.run();
	
}
