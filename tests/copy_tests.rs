use file_manager::{prelude::*, TempDir};

#[test]
fn copy_test() -> Result<(), String> {
    let tmp = TempDir::new().unwrap();
    let base = tmp.join("base").make_dirs()?;

    let src = base.join("source").make_dirs()?;
    src.join("file1.txt").write().string("file1")?;
    src.join("file2.txt").write().string("file2")?;

    let dest = base.join("destination").make_dirs()?;

    src.dir_copy()
        .filter_include("**/file1.txt")?
        .filter_exclude("**/file2.txt")?
        .to_dir(&dest)?;

    insta::assert_snapshot!(base.dir_list().to_ascii().unwrap(), @r###"
    base
     ├─ source
     │   ├─ file2.txt
     │   └─ file1.txt
     └─ destination
    "###);

    Ok(())
}
