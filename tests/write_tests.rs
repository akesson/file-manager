use file_manager::{prelude::*, TempDir};

#[test]
fn read_write_string_roundtrip() -> Result<(), String> {
    let tmp = TempDir::new()?;

    let dir = tmp.join("myfolder").make_dirs()?;

    let file = dir.join("myfile.txt").write().string("hello world")?;

    let text = file.read().string()?;

    assert_eq!(text, "hello world");
    Ok(())
}

#[test]
fn delete() -> Result<(), String> {
    let tmp = TempDir::new()?;
    let dir = tmp.join("base").make_dirs()?;

    dir.join("dir-1/dir-1.2").make_dirs()?;
    dir.join("dir-1/dir-1.2/file-0.txt").write().string("")?;
    dir.join("dir-2").make_dirs()?;
    dir.join("dir-2/file-1.txt").write().string("").unwrap();
    dir.join("dir-2/file-2.txt").write().string("")?;

    insta::assert_snapshot!(dir.dir_list().ascii()?, @r###"
    base
     ├─ dir-2
     │   ├─ file-2.txt
     │   └─ file-1.txt
     └─ dir-1
         └─ dir-1.2
             └─ file-0.txt
    "###);

    dir.join("dir-1").delete().dir().all()?;

    insta::assert_snapshot!(dir.dir_list().ascii()?, @r###"
    base
     └─ dir-2
         ├─ file-2.txt
         └─ file-1.txt
    "###);

    dir.join("dir-2/file-2.txt").delete().file()?;
    dir.join("dir-2/file-1.txt").delete().file()?;

    insta::assert_snapshot!(dir.dir_list().ascii()?, @r###"
    base
     └─ dir-2
    "###);

    dir.join("dir-2").delete().dir().empty()?;

    insta::assert_snapshot!(dir.dir_list().ascii()?, @r###"
    base
    "###);

    let res = dir.join("inexistant").delete().dir().empty();

    let err_str = res.err().map(|e| e.to_string()).unwrap_or_default();

    // the error message looks like this, but since it contains the random tmp dir path, it's not stable:
    //
    // insta::assert_snapshot!(err_str, @r###"
    // Error: could not delete directory /var/folders/wr/cghr03c56bq3s5sg4_s72pm00000gn/T/.tmpdMsR4D/base/inexistant
    //        ↳ No such file or directory (os error 2)
    // "###);

    assert!(err_str.starts_with("Error: could not delete directory"));
    assert!(err_str.ends_with("No such file or directory (os error 2)\n"));
    Ok(())
}
