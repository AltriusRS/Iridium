use clap::{Arg, App};
use comrak::{markdown_to_html, ComrakOptions, ComrakExtensionOptions};
use std::fs::*;
use std::io::{Read, Write};
use std::path::PathBuf;

const HEADERS: &str = "<link rel=\"stylesheet\" href=\"http://github.com/yrgoldteeth/darkdowncss/raw/master/darkdown.css\" />\n<link rel=\"stylesheet\" href=\"//cdnjs.cloudflare.com/ajax/libs/highlight.js/10.1.2/styles/default.min.css\">\n<script src=\"//cdnjs.cloudflare.com/ajax/libs/highlight.js/10.1.2/highlight.min.js\"></script>\n<script>hljs.initHighlightingOnLoad();</script>";

fn main() {
    let matches = App::new("Iridium")
        .version("0.3")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("A Static Site Generator")
        .arg(Arg::with_name("in")
            .short("i")
            .long("input")
            .value_name("PATH")
            .help("Sets the location to read from. (Can be a file or directory)")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("out")
            .short("o")
            .long("output")
            .value_name("PATH")
            .help("Sets the location to write to. (Must be a directory)")
            .takes_value(true)
            .required(true))
        .get_matches();

    if matches.is_present("in") && matches.is_present("out") {
        let input = matches.value_of("in").unwrap();
        let output = matches.value_of("out").unwrap();
        let metadata = metadata(input);
        if metadata.is_ok() {
            let md = metadata.unwrap();
            if md.is_dir() {
                let paths = read_directory(input);
                println!("Found: {} files to process", paths.len());
                let processes = handle_non_md(paths, input, output);
                println!("Final Queue: {} files", processes.len());
            } else {
                println!("")
                // read_file();
            }
        } else {
            println!("An error occurred: {:#?}", metadata.unwrap_err());
            std::process::exit(1);
        }
    } else {}
}

fn read_directory(path: &str) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let files = read_dir(path).unwrap();
    for f in files {
        if f.is_ok() {
            let file = f.unwrap();
            if file.metadata().unwrap().is_dir() {
                let mut p1 = read_directory(file.path().to_str().unwrap());
                paths.append(&mut p1);
            } else {
                paths.push(file.path())
            }
        } else {
            println!("Unable to open: {:#?}", f.unwrap_err())
        }
    }
    paths
}

fn handle_non_md(paths: Vec<PathBuf>, base: &str, out: &str) -> Vec<PathBuf> {
    let mut processes = Vec::<PathBuf>::new();
    for path in paths {
        let pathstr = path.to_str().unwrap();
        if pathstr.ends_with(".md") {
            processes.push(path);
        } else {
            let mut root = String::from(base);
            if !root.ends_with("/") {
                root = format!("{}/", root)
            }
            let mut destination = String::from(out);
            if !destination.ends_with("/") {
                destination = format!("{}/", destination)
            }
            let mut tiers = pathstr.split(root.as_str()).collect::<Vec<&str>>();
            let final_path = format!("{}{}", destination, tiers.pop().unwrap());
            let mut meta = metadata(&destination);
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

fn make_file(path: String, root: String) {
    println!("{}\t{}", path, root);
    let meta = metadata(&path);
    let mut content = Vec::<u8>::new();
    let available = File::open(root);
    if available.is_ok() {
        available.unwrap().read_to_end(&mut content);
    } else {
        println!("{}", available.unwrap_err().to_string());
    }

    if meta.is_ok() {
        remove_file(&path);
        let mut file = File::create(&path).unwrap();
        file.write(content.as_mut());
    } else {
        println!("{:#?}", meta);
        let err = meta.unwrap_err();
        if err.kind() == std::io::ErrorKind::NotFound {
            let mut file = File::create(&path).unwrap();
            file.write(content.as_mut());
        }
    }
}

fn read_file(path: String, out_path: String) {
    // Read the file
    let mut content_file = File::open(path).unwrap();
    let mut content = String::new();
    content_file.read_to_string(&mut content).unwrap();

    // Read the theme's stylesheet
    let mut css: String = String::new();
    let mut res = File::open("./dark.css").unwrap();
    let read = res.read_to_string(&mut css).unwrap();

    // Begin parsing MD to HTML
    let mut html = format!("<!DOCTYPE html>\n<html>\n<head>\n{}\n<style>{}</style>", HEADERS, css);
    html = format!("{}\n</head>\n<body>\n<div class=\"container\">", html);
    html = format!("{}\n{}\n</div>\n</body>\n<html>", html, markdown_to_html(&content.as_str(), &ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: None,
            footnotes: true,
            description_lists: true,
        },
        parse: Default::default(),
        render: Default::default(),
    }));

    let mut out: File;
    let meta = metadata(&out_path);
    if meta.is_ok() {
        let trydel = remove_file(&out_path);
        if trydel.is_ok() {
            out = File::create(&out_path).unwrap();
            write(html, &mut out)
        } else {
            println!("Failed to parse \"{}\".\nReason: {:#?}", &out_path, trydel.unwrap_err());
        }
    } else {
        out = File::create(&out_path).unwrap();
        write(html, &mut out)
    }
}

fn write(content: String, file: &mut File) {
    file.write(content.as_bytes());
    println!("Done");
}