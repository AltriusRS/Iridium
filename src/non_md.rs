use std::path::PathBuf;
use std::fs::{metadata, create_dir_all, File, remove_file};
use std::io::{Write, Read};

pub(crate) fn handle_non_md(paths: Vec<PathBuf>, base: &str, out: &str) -> Vec<(String, String)> {
    let mut processes = Vec::<(String, String)>::new();
    for path in paths {
        let mut root = String::from(base);
        if !root.ends_with("/") {
            root = format!("{}/", root)
        }
        let mut destination = String::from(out);
        if !destination.ends_with("/") {
            destination = format!("{}/", destination)
        }

        let pathstr = path.to_str().unwrap();
        let p2 = pathstr.clone();
        let mut tiers = p2.split(root.as_str()).collect::<Vec<&str>>();
        let final_path = format!("{}{}", destination, tiers.pop().unwrap());

        if pathstr.ends_with(".md") || pathstr.ends_with(".markdown") {
            processes.push((pathstr.to_string(), final_path));
        } else {
            let meta = metadata(&destination);
            if meta.is_ok() {
                make_file(final_path, pathstr.to_string());
            } else {
                let err = meta.unwrap_err();
                if err.kind() == std::io::ErrorKind::NotFound {
                    create_dir_all(destination);
                    make_file(final_path, pathstr.to_string());
                }
            }
        }
    }
    return processes;
}

pub(crate) fn make_file(path: String, root: String) {
    let meta = metadata(&path);
    let mut content = Vec::<u8>::new();
    let available = File::open(root);

    if available.is_ok() {
        available.unwrap().read_to_end(&mut content);
    } else {
        println!("Error - File unavailable: {}", available.unwrap_err().to_string());
    }


    if meta.is_ok() {
        remove_file(&path);
        let mut file = File::create(&path).unwrap();
        file.write(content.as_mut());
        println!("Migrated: {}", path);
    } else {
        let err = meta.unwrap_err();
        if err.kind() == std::io::ErrorKind::NotFound {
            let op = path.clone();
            let mut entries = op.split("/").collect::<Vec<&str>>();
            let _ = entries.pop();
            let dir_path = entries.join("/");
            create_dir_all(dir_path);
            let mut file = File::create(&path).unwrap();
            file.write(content.as_mut());
            println!("Migrated: {}", path);
        }
    }
}