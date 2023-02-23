use std::fmt;
use std::fmt::Write;

#[derive(Clone)]
pub enum Tree {
    Node(String, Vec<Tree>),
    Leaf(String),
}

impl Tree {
    pub fn node(name: impl Into<String>, children: Vec<Tree>) -> Self {
        Self::Node(name.into(), children)
    }

    pub fn leaf(name: impl Into<String>) -> Self {
        Self::Leaf(name.into())
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_tree(f, self)
    }
}

pub fn write_tree(f: &mut dyn Write, tree: &Tree) -> fmt::Result {
    write_tree_element(f, tree, &vec![])
}

fn write_tree_element(f: &mut dyn Write, tree: &Tree, level: &Vec<usize>) -> fmt::Result {
    use Tree::*;
    const EMPTY: &str = "    ";
    const EDGE: &str = " └─";
    const PIPE: &str = " │  ";
    const BRANCH: &str = " ├─";

    let maxpos = level.len();
    let mut second_line = String::new();
    for (pos, l) in level.iter().enumerate() {
        let last_row = pos == maxpos - 1;
        if *l == 1 {
            if !last_row {
                write!(f, "{}", EMPTY)?
            } else {
                write!(f, "{}", EDGE)?
            }
            second_line.push_str(EMPTY);
        } else {
            if !last_row {
                write!(f, "{}", PIPE)?
            } else {
                write!(f, "{}", BRANCH)?
            }
            second_line.push_str(PIPE);
        }
    }
    match tree {
        Node(title, children) => {
            let mut d = children.len();
            if level.len() == 0 {
                write!(f, "{}\n", title)?;
            } else {
                write!(f, " {}\n", title)?;
            }

            for s in children {
                let mut lnext = level.clone();
                lnext.push(d);
                d -= 1;
                write_tree_element(f, s, &lnext)?;
            }
        }
        Leaf(line) => writeln!(f, " {}", line)?,
    }
    Ok(())
}
