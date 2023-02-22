<!-- ASCII TREE START -->
```text
Utf8Path
 ├─ Directory operations
 │   ├─ .dir_list()
 │   │   ├─ options: ListOptions
 │   │   ├─ .to_ascii()
 │   │   ├─ .to_paths()
 │   │   ├─ .to_vec(fn)
 │   │   └─ .into_iter()
 │   ├─ .dir_copy()
 │   │   ├─ options: ListOptions & FilterOptions & DestinationOptions
 │   │   └─ .to_dir()
 │   ├─ .dir_zip()
 │   │   ├─ options: .compression_level() + ListOptions & FilterOptions
 │   │   └─ .to_file()
 │   ├─ .dir_move()
 │   ├─ .dir_delete()
 │   └─ .dir_create()
 ├─ .resolve()
 └─ File operations
     ├─ .file_write()
     │   ├─ options: DestinationOptions
     │   ├─ output:  .json(), .string(), .bytes()
     │   └─ .zip()
     │       ├─ options: .compression_level()
     │       └─ output:  .json(), .string(), .bytes()
     └─ .file_read()
         ├─ .json<T>()? -> T
         ├─ .string()? -> String
         ├─ .bytes()? -> Vec<u8>
         └─ .unzip()
             ├─ options: FilterOptions & DestinationOptions
             ├─ .to_dir()?
             └─ .list()?

ListOptions
 └─ .min_depth(), .max_depth(), .follow_links(), .same_file_system()

FilterOptions
 ├─ .filter_include(glob)?
 ├─ .filter_exclude(glob)?
 └─ .filter(fn(path) -> bool)?

DestinationOptions
 ├─ .destination_map(fn(path) -> path)?
 ├─ .destination( Existing::{Overwrite, Append, Skip, Error} )
 └─ .destination( Missing::{Create, Skip, Error} )

```
<!-- ASCII TREE END -->
