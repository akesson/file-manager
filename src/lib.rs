mod actions;
mod build;
mod dir;
mod error;
mod helpers;

#[cfg(feature = "tempdir")]
pub use helpers::tempdir::TempDir;

// re-exports
pub use camino::{Utf8Path, Utf8PathBuf};

pub use actions::{PathDelete, PathRead, PathResolve, PathWrite};
pub use error::FileManagerError;

use helpers::env;

pub type Result<T> = std::result::Result<T, FileManagerError>;

pub mod prelude {
    pub use crate::actions::{PathBase, PathDelete, PathRead, PathResolve, PathWrite};
    pub use crate::dir::{DirCopy, DirList};
}

#[cfg(feature = "ascii")]
#[test]
fn generate_function_hierarchy() {
    use crate::helpers::Tree;
    use camino::Utf8PathBuf;

    let trees = vec![
        Tree::node(
            "Utf8Path",
            vec![
                Tree::node(
                    "Directory operations",
                    vec![
                        Tree::node(
                            ".dir_list()",
                            vec![
                                Tree::leaf("options: ListOptions"),
                                Tree::leaf(".to_ascii()"),
                                Tree::leaf(".to_paths()"),
                                Tree::leaf(".to_vec(fn)"),
                                Tree::leaf(".into_iter()"),
                            ],
                        ),
                        Tree::node(
                            ".dir_copy()",
                            vec![
                                Tree::leaf(
                                    "options: ListOptions & FilterOptions & DestinationOptions",
                                ),
                                Tree::leaf(".to_dir()"),
                            ],
                        ),
                        Tree::node(
                            ".dir_zip()",
                            vec![
                                Tree::leaf(
                                    "options: .compression_level() + ListOptions & FilterOptions",
                                ),
                                Tree::leaf(".to_file()"),
                            ],
                        ),
                        Tree::node(".dir_move()", vec![]),
                        Tree::node(".dir_delete()", vec![]),
                        Tree::leaf(".dir_create()"),
                    ],
                ),
                Tree::node(".resolve()", vec![]),
                Tree::node(
                    "File operations",
                    vec![
                        Tree::node(
                            ".file_write()",
                            vec![
                                Tree::leaf("options: DestinationOptions"),
                                Tree::leaf("output:  .json(), .string(), .bytes()"),
                                Tree::node(
                                    ".zip()",
                                    vec![
                                        Tree::leaf("options: .compression_level()"),
                                        Tree::leaf("output:  .json(), .string(), .bytes()"),
                                    ],
                                ),
                            ],
                        ),
                        Tree::node(
                            ".file_read()",
                            vec![
                                Tree::leaf(".json<T>()? -> T"),
                                Tree::leaf(".string()? -> String"),
                                Tree::leaf(".bytes()? -> Vec<u8>"),
                                Tree::node(
                                    ".unzip()",
                                    vec![
                                        Tree::leaf("options: FilterOptions & DestinationOptions"),
                                        Tree::leaf(".to_dir()?"),
                                        Tree::leaf(".list()?"),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        ),
        Tree::node(
            "ListOptions",
            vec![Tree::leaf(
                ".min_depth(), .max_depth(), .follow_links(), .same_file_system()",
            )],
        ),
        Tree::node(
            "FilterOptions",
            vec![
                Tree::leaf(".filter_include(glob)?"),
                Tree::leaf(".filter_exclude(glob)?"),
                Tree::leaf(".filter(fn(path) -> bool)?"),
            ],
        ),
        Tree::node(
            "DestinationOptions",
            vec![
                Tree::leaf(".destination_map(fn(path) -> path)?"),
                Tree::leaf(".destination( Existing::{Overwrite, Append, Skip, Error} )"),
                Tree::leaf(".destination( Missing::{Create, Skip, Error} )"),
            ],
        ),
    ];

    let mut readme = Utf8PathBuf::from("./README.md").read().string().unwrap();

    const SECT_START: &str = "<!-- ASCII TREE START -->\n";
    const SECT_END: &str = "\n<!-- ASCII TREE END -->";

    let start = readme.find(SECT_START).unwrap() + SECT_START.len();
    let end = readme.find(SECT_END).unwrap();

    let ascii_tree = format!(
        "```text\n{}\n```",
        trees
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    );
    if readme[start..end] != ascii_tree {
        readme.replace_range(start..end, &ascii_tree);
        Utf8PathBuf::from("./README.md")
            .write()
            .string(&readme)
            .unwrap();
    }
}
