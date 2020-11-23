use clap::{Arg, App};
use std::fs::*;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};
use std::str::FromStr;
use crate::parser::{parse, relink, parse_theme};
use crate::non_md::handle_non_md;

mod parser;
mod non_md;

const HEADERS: &str = "<script src=\"//cdnjs.cloudflare.com/ajax/libs/highlight.js/10.1.2/highlight.min.js\"></script>\n<script>hljs.initHighlightingOnLoad();</script>";

fn main() {
    let matches = App::new("Iridium")
        .version("0.1.0")
        .author("Thomas B. <tom.b.2k2@gmail.com>")
        .about("A static site generator for the modern era.")
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
        .arg(Arg::with_name("theme")
            .long("theme")
            .short("t")
            .long_help("Selects the theme to use in the rendering process (Default: 'Iridium')\nAvailable themes:\nIridium\nIridium-light\nNoir\nNeon - Produced by Jas777 on GitHub")
            .takes_value(true))
        .arg(Arg::with_name("ignore")
            .long("ignore")
            .help("Path to either a .gitignore or a .iridium file")
            .takes_value(true))
        .get_matches();

    let mut ignore = ".iridium";
    if matches.is_present("ignore") {
        ignore = matches.value_of("ignore").unwrap();
    }

    let mut theme = "Iridium";
    if matches.is_present("theme") {
        theme = matches.value_of("theme").unwrap();
    }

    let mut wm = true;
    if matches.is_present("watermark") {
        wm = false;
    }

    if matches.is_present("in") && matches.is_present("out") {
        let input = matches.value_of("in").unwrap();
        let output = matches.value_of("out").unwrap();
        let p = Path::new(input).canonicalize();
        if p.is_ok() {
            let pa = p.unwrap();
            let metadata = metadata(pa);
            if metadata.is_ok() {
                let md = metadata.unwrap();
                if md.is_dir() {
                    println!("Discovering Files...");
                    let mut paths = read_directory(input);
                    let mut tot: isize = paths.len() as isize;
                    println!("Discovered {} Files", tot);
                    paths = non_md::filter(paths, input, ignore);
                    let t1: isize = tot;
                    tot = paths.len() as isize;
                    println!("Ignoring {} files", t1 - tot);
                    println!("Migrating incompatible files...");
                    let processes = handle_non_md(paths, input, output);
                    let mut ptot: isize = processes.len() as isize;
                    let mut index: isize = tot - ptot;

                    for (source, destination) in processes {
                        read_file(source, destination, wm, theme);
                        index += 1;
                    }
                    println!("Migrated {} Files", index);
                    println!("Compiled {} Files", ptot);
                } else {
                    let mut nodes = input.clone().split("/").collect::<Vec<&str>>();
                    let file = nodes.pop();
                    let mut root: String = nodes.join("/");

                    let mut destination = String::from(output);
                    if !destination.ends_with("/") {
                        destination = format!("{}/", destination)
                    }

                    let pathstr = input.clone();
                    let p2 = pathstr.clone();
                    let mut tiers = p2.split(root.as_str()).collect::<Vec<&str>>();
                    let final_path = format!("{}{}", destination, tiers.pop().unwrap());
                    read_file(pathstr.to_string(), final_path, wm, theme);
                }
                println!("Compilation complete.");
            } else {
                println!("An error occurred (step 1): {:#?}", metadata.unwrap_err());
                std::process::exit(1);
            }
        } else {
            println!("Failed to canonicalize {}\n{:#?}", input, p.unwrap_err())
        }
    }
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

fn read_file(path: String, out_path: String, wm: bool, theme: &str) {
    // Read the file
    let mut content_file = File::open(&path).unwrap();
    let mut content = String::new();
    content_file.read_to_string(&mut content).unwrap();

    // Read the theme's stylesheet
    let mut css: &str = parse_theme(theme);

    let mut watermark = String::from("<div style=\"text-align: center; padding: 1em; color: #aaa\"><h4>Powered by <a href=\"https://github.com/fatalcenturion/Iridium\">Iridium</a><h4></div>");
    if !wm {
        watermark = String::new();
    }
    // Begin parsing MD to HTML
    let mut html = format!("<!DOCTYPE html>\n<html>\n<head>\n{}\n<style>{}</style>", HEADERS, css);
    html = format!("{}\n<script src=\"https://ajax.googleapis.com/ajax/libs/jquery/3.5.1/jquery.min.js\"></script>\n</head>\n<body>\n<div class=\"container\">", html);
    html = format!("{}\n{}\n</div>\n{}\n<script>{}</script></body>\n<html>", html, parse(content), watermark, "
    document.addEventListener('DOMContentLoaded', function() {
	    document.querySelectorAll(\"h1, h2, h3, h4, h5, h6\").forEach(element => {
		    element.innerHTML += `<div class=\"anchor\" style=\"display:none;\" id=\"${(element.innerText.toLowerCase().replace(/[^\\w]/gmi, \"\")).split(\" \").join(\"-\")}\">Anchor point</div>`;
		    element.children[0].onclick = () => {let em = document.getElementById(window.location.hash.split(/#|\\?[^\\s]*/g).join(\"\")); console.log(`Navigating to: ${window.location.hash.split(/#|\\?[^\\s]*/g).join(\"\")}`); if(em !== null && em !== undefined) em.parentElement.scrollIntoView({ behavior: 'instant', block: 'start' })}
		    })
	let em = document.getElementById(window.location.hash.replace(/#|\\?[^\\s]*/g, \"\"));
	console.log(`Navigating to: ${window.location.hash.replace(/#|\\?[^\\s]*/g, \"\")}`);
	if(em !== null && em !== undefined) em.parentElement.scrollIntoView({ behavior: 'instant', block: 'start' })
}, false);");

    html = relink(html, "html");

    let mut out: File;
    let meta = metadata(&out_path);
    let op = out_path.clone();
    let mut entries = op.split("/").collect::<Vec<&str>>();
    let name = entries.pop().unwrap().replace(".md", "").replace(".markdown", "");
    let dir_path = entries.join("/");
    if meta.is_ok() {
        let trydel = remove_file(&out_path);
        if trydel.is_ok() {
            println!("Compiled: {}", &out_path.replace(".md", ".html").replace(".markdown", ".html"));
            out = File::create(&out_path.replace(".md", ".html").replace(".markdown", ".html")).unwrap();
            write(html, &mut out)
        } else {
            println!("Failed to get metadata for \"{}\".\nReason: {:#?}", out_path, trydel.unwrap_err());
        }
    } else {
        create_dir_all(dir_path);
        println!("Compiled: {}", &out_path.replace(".md", ".html").replace(".markdown", ".html"));
        out = File::create(&out_path.replace(".md", ".html").replace(".markdown", ".html")).unwrap();
        write(html, &mut out)
    }
}

fn write(content: String, file: &mut File) {
    file.write(content.as_bytes());
}

