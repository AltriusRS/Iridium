use std::path::{PathBuf, Path};
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

pub(crate) fn filter(paths: Vec<PathBuf>, src: &str, rule_file: &str) -> Vec<PathBuf> {
    let mut p2: Vec<PathBuf> = Vec::new();
    let mut p: String = src.to_string();
    if !p.ends_with("/") {
        p = format!("{}/", p);
    }
    if rule_file != ".iridium" {
        p = format
    } else {
        p = format!("{}{}", p, rule_file);
    }

    let mut rule_string = String::new();

    let f = File::open(p);

    let mut ignore = gitignored::Gitignore::new(src, false, true);

    return if f.is_ok() {
        f.unwrap().read_to_string(&mut rule_string);
        let liners = rule_string.split("\r").collect::<Vec<&str>>();
        let l2 = liners.join("");
        let mut lines = l2.split("\n").collect::<Vec<&str>>();

        // filter paths
        for path in paths {
            if !ignore.ignores(&*lines, &path) {
                if path.to_str().unwrap().contains(".iridium") {
                    println!("Ignoring: {:#?} ", path);
                } else {
                    p2.push(path);
                }
            } else {
                println!("Ignoring: {:#?} ", path);
            }
        }
        p2
    } else {
        paths
    };
}