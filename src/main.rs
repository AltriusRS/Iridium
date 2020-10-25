use regex::Regex;
use scraper::{Html, Selector};
use clap::{Arg, App};
use comrak::{markdown_to_html, ComrakOptions, ComrakExtensionOptions, ComrakParseOptions, ComrakRenderOptions};
use std::fs::*;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::ops::{Range, RangeInclusive};

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
        .arg(Arg::with_name("watermark")
            .long("no-water-mark")
            .help("Removes the watermark")
            .takes_value(false))
        .get_matches();
    let mut wm = true;
    if matches.is_present("watermark") {
        wm = false;
    }
    if matches.is_present("in") && matches.is_present("out") {
        let input = matches.value_of("in").unwrap();
        let output = matches.value_of("out").unwrap();
        let metadata = metadata(input);
        if metadata.is_ok() {
            let md = metadata.unwrap();
            if md.is_dir() {
                println!("Discovering Files...");
                let paths = read_directory(input);
                let tot = paths.len();
                println!("Discovered {} Files", tot);
                println!("Migrating incompatible files...");
                let processes = handle_non_md(paths, input, output);
                let mut index = tot - processes.len();
                let ptot = processes.len();
                println!("Migrated {} Files", index);
                let mut progress: f32 = ((100 / tot) * index) as f32;

                for (source, destination) in processes {
                    print!("\rCompiling: {}%", progress);
                    read_file(source, destination, wm);
                    index += 1;
                    progress = ((100 / tot) * index) as f32;
                }
                println!("\rCompiling: 100%\nCompiled {} Files", ptot);
            } else {
                println!("")
                // read_file();
            }
        } else {
            println!("An error occurred (step 1): {:#?}", metadata.unwrap_err());
            std::process::exit(1);
        }
    } else {}
    println!("Compilation complete.")
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

fn handle_non_md(paths: Vec<PathBuf>, base: &str, out: &str) -> Vec<(String, String)> {
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

        if pathstr.ends_with(".md") {
            processes.push((pathstr.to_string(), final_path));
        } else {
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
        }
    }
}

fn read_file(path: String, out_path: String, wm: bool) {
    // Read the file
    let mut content_file = File::open(path).unwrap();
    let mut content = String::new();
    content_file.read_to_string(&mut content).unwrap();

    // Read the theme's stylesheet
    let mut css: String = String::new();
    let mut res = File::open("./dark.css").unwrap();
    let read = res.read_to_string(&mut css).unwrap();

    let mut watermark = String::from("<div style=\"text-align: center; padding: 1em; color: #aaa\"><h4>Powered by <a href=\"https://github.com/fatalcenturion/Iridium\">Iridium</a><h4></div>");
    if !wm {
        watermark = String::new();
    }
    // Begin parsing MD to HTML
    let mut html = format!("<!DOCTYPE html>\n<html>\n<head>\n{}\n<style>{}</style>", HEADERS, css);
    html = format!("{}\n<script src=\"https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js\"></script>\n</head>\n<body>\n<div class=\"container\">", html);
    html = format!("{}\n{}\n</div>{}\n{}</body>\n<html>", html, markdown_to_html(&content.as_str(), &ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: false,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: None,
            footnotes: true,
            description_lists: true,
        },
        parse: ComrakParseOptions {
            smart: false,
            default_info_string: None,
        },
        render: ComrakRenderOptions {
            hardbreaks: false,
            github_pre_lang: false,
            width: 0,
            unsafe_: true,
            escape: false,
        },
    }), "<script>$(document).ready(function(){$(\"a\").on(\"click\",function(o){if(\"\"!==this.hash){o.preventDefault();var t=this.hash;$(\"html, body\").animate({scrollTop:$(t).offset().top},800,function(){window.location.hash=t})}})});</script>", watermark);

    let matcher = Regex::new(r"<\s*a[^>]*>").unwrap();
    let htclone = html.clone();
    let matches = matcher.find_iter(&htclone).collect::<Vec<regex::Match>>();
    for token in matches {
        let start = token.start();
        let end = token.end();
        let content = token.as_str();
        let fragment = Html::parse_fragment(content);
        let body = html.split(content).collect::<Vec<&str>>();
        let slice = fragment.select(&Selector::parse(r#"a"#).unwrap()).next().unwrap();
        let mut href: String = String::from(slice.value().attr("href").unwrap());
        if href.starts_with("//") || href.starts_with("http") {
            // ignore the link
        } else {
            if href.contains('#') {
                let collection = href.split('#').collect::<Vec<&str>>();
                let sref: String = collection.join(".html#");
                href = sref;
            } else {
                href = format!("{}.html", href);
            }
            html = body.join(format!("<a href=\"{}\">", href).as_str())
        }
    }
    let mut out: File;
    let meta = metadata(&out_path);
    if meta.is_ok() {
        let trydel = remove_file(&out_path);
        if trydel.is_ok() {
            out = File::create(&out_path.replace(".md", ".html")).unwrap();
            write(html, &mut out)
        } else {
            println!("Failed to parse \"{}\".\nReason: {:#?}", &out_path, trydel.unwrap_err());
        }
    } else {
        let op = out_path.clone();
        let mut entries = op.split("/").collect::<Vec<&str>>();
        let _ = entries.pop();
        let dir_path = entries.join("/");
        create_dir_all(dir_path);
        out = File::create(&out_path.replace(".md", ".html")).unwrap();
        write(html, &mut out)
    }
}

fn write(content: String, file: &mut File) {
    file.write(content.as_bytes());
}