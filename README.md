<!-- ASCII TREE START -->
```text
Utf8Path
 ├─ .list()
 │   └─ .ascii()
 ├─ .copy()
 ├─ .move()
 ├─ .delete()
 ├─ .resolve()
 ├─ .write()
 │   ├─ options: .no_overwrite()
 │   ├─ output:  .json(), .string(), .bytes()
 │   └─ .zip()
 │       ├─ options: .compression_level()
 │       └─ output:  .json(), .string(), .bytes()
 ├─ .read()
 │   ├─ .json<T>() -> Result<T>
 │   ├─ .string() -> Result<String>
 │   ├─ .bytes() -> Result<Vec<u8>>
 │   └─ .extract()
 │       ├─ .to(AsRef<Path>)
 │       └─ .unzip()
 └─ .make_dirs()

```
<!-- ASCII TREE END -->
