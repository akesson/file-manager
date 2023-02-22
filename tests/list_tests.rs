use file_manager::{prelude::*, TempDir, Utf8Path};

#[test]
fn list_all() -> Result<(), String> {
    let tmp = TempDir::new().unwrap();

    let dir = tmp.join("myfolder").make_dirs()?;

    let workspace = dir.join("workspace").make_dirs()?;
    workspace.join("Cargo.toml").write().string("")?;
    workspace.join("Cargo.lock").write().string("")?;
    workspace.join("README.md").write().string("")?;
    workspace.join("LICENSE.md").write().string("")?;
    workspace
        .join(".gitignore")
        .write()
        .string("target/\nCargo.lock")?;

    populate_lib_dirs(&workspace, "my_crate_1", "mymod1")?;
    populate_lib_dirs(&workspace, "my_crate_2", "mymod2")?;

    insta::assert_snapshot!(dir.dir_list().to_ascii().unwrap(), @r###"
    myfolder
     └─ workspace
         ├─ my_crate_1
         │   ├─ Cargo.toml
         │   └─ src
         │       ├─ mymod1
         │       │   └─ mod.rs
         │       └─ lib.rs
         ├─ Cargo.toml
         ├─ LICENSE.md
         ├─ target
         │   ├─ release
         │   │   ├─ my_crate_2.a
         │   │   └─ my_crate_1.a
         │   └─ debug
         │       ├─ my_crate_2.a
         │       └─ my_crate_1.a
         ├─ Cargo.lock
         ├─ README.md
         ├─ my_crate_2
         │   ├─ Cargo.toml
         │   └─ src
         │       ├─ lib.rs
         │       └─ mymod2
         │           └─ mod.rs
         └─ .gitignore
    "###);

    let found = dir
        .dir_list()
        .filter_exclude("**/my_crate_1.a")?
        .filter_include("**/target/**")?
        .into_iter()
        .map(|x| x.unwrap().relative_path().to_string())
        .collect::<Vec<_>>()
        .join("\n");

    insta::assert_snapshot!(found, @r###"
    workspace/target/release
    workspace/target/release/my_crate_2.a
    workspace/target/debug
    workspace/target/debug/my_crate_2.a
    "###);

    let found = dir
        .dir_list()
        .filter_include("*.rs")?
        .to_vec(|x| format!("{} {}", x.depth(), x.relative_path()))?
        .join("\n");

    insta::assert_snapshot!(found, @r###"
    5 workspace/my_crate_1/src/mymod1/mod.rs
    4 workspace/my_crate_1/src/lib.rs
    4 workspace/my_crate_2/src/lib.rs
    5 workspace/my_crate_2/src/mymod2/mod.rs
    "###);
    Ok(())
}

fn populate_lib_dirs(workspace: &Utf8Path, name: &str, module: &str) -> Result<(), String> {
    let dir = workspace.join(name).make_dirs()?;
    dir.join("Cargo.toml").write().string("")?;
    dir.join("src").make_dirs()?;
    dir.join("src/lib.rs").write().string("")?;
    dir.join(format!("src/{module}")).make_dirs()?;
    dir.join(format!("src/{module}/mod.rs"))
        .write()
        .string("")?;

    let lib = name.replace('-', "_");

    workspace.join("target/debug").make_dirs()?;
    workspace
        .join(format!("target/debug/{lib}.a"))
        .write()
        .string("")?;

    workspace.join("target/release").make_dirs()?;
    workspace
        .join(format!("target/release/{lib}.a"))
        .write()
        .string("")?;
    Ok(())
}
