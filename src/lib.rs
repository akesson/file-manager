mod actions;
mod error;
mod helpers;
mod proto;

#[cfg(feature = "tempdir")]
pub use helpers::tempdir::TempDir;

// re-exports
pub use camino::{Utf8Path, Utf8PathBuf};

pub use error::FileManagerError;

pub use actions::{PathCopy, PathDelete, PathList, PathRead, PathResolve, PathWrite};
use helpers::env;

pub type Result<T> = std::result::Result<T, FileManagerError>;

pub mod prelude {
    pub use crate::actions::{
        PathBase, PathCopy, PathDelete, PathList, PathRead, PathResolve, PathWrite,
    };
}

#[cfg(feature = "ascii")]
#[test]
fn generate_function_hierarchy() {
    use crate::helpers::Tree;
    use camino::Utf8PathBuf;

    let tree = Tree::node(
        "Utf8Path",
        vec![
            Tree::node(".list()", vec![Tree::node(".ascii()", vec![])]),
            Tree::node(".copy()", vec![]),
            Tree::node(".move()", vec![]),
            Tree::node(".delete()", vec![]),
            Tree::node(".resolve()", vec![]),
            Tree::node(
                ".write()",
                vec![
                    Tree::leaf("options: .no_overwrite()"),
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
                ".read()",
                vec![
                    Tree::leaf(".json<T>() -> Result<T>"),
                    Tree::leaf(".string() -> Result<String>"),
                    Tree::leaf(".bytes() -> Result<Vec<u8>>"),
                    Tree::node(
                        ".extract()",
                        vec![Tree::leaf(".to(AsRef<Path>)"), Tree::leaf(".unzip()")],
                    ),
                ],
            ),
            Tree::leaf(".make_dirs()"),
        ],
    );

    let mut readme = Utf8PathBuf::from("./README.md").read().string().unwrap();

    const SECT_START: &str = "<!-- ASCII TREE START -->\n";
    const SECT_END: &str = "\n<!-- ASCII TREE END -->";

    let start = readme.find(SECT_START).unwrap() + SECT_START.len();
    let end = readme.find(SECT_END).unwrap();

    let ascii_tree = format!("```text\n{tree}\n```",);
    if readme[start..end] != ascii_tree {
        readme.replace_range(start..end, &ascii_tree);
        Utf8PathBuf::from("./README.md")
            .write()
            .string(&readme)
            .unwrap();
    }
}
