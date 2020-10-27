use regex::Regex;
use scraper::{Html, Selector};
use clap::{Arg, App};
use comrak::{markdown_to_html, ComrakOptions, ComrakExtensionOptions, ComrakParseOptions, ComrakRenderOptions};
use std::fs::*;
use wkhtmltopdf::*;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};
use std::str::FromStr;

const HEADERS: &str = "<script src=\"//cdnjs.cloudflare.com/ajax/libs/highlight.js/10.1.2/highlight.min.js\"></script>\n<script>hljs.initHighlightingOnLoad();</script>";

fn main() {
    let mut pdf_app = PdfApplication::new().expect("Failed to init PDF application");
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
        .arg(Arg::with_name("pdf")
            .long("pdf")
            .help("Renders the output as PDF content, instead of the normal HTML")
            .takes_value(false))
        .arg(Arg::with_name("pdf-mirror")
            .long("pdf-mirror")
            .help("Renders the output as PDF content, as well as the normal HTML")
            .takes_value(false))
        .get_matches();

    let mut wm = true;
    if matches.is_present("watermark") {
        wm = false;
    }

    let mut pdf = false;
    if matches.is_present("pdf") {
        pdf = true;
    }

    let mut pdfm = false;
    if matches.is_present("pdf-mirror") {
        pdfm = true;
    }

    if pdf && !pdfm {
        println!("PDF Mode")
    } else if !pdf && pdfm {
        println!("PDF Mirror Mode")
    } else if pdf && pdfm {
        pdf = false;
        println!("PDF Mirror Mode")
    } else {
        println!("HTML Mode")
    }

    if pdf || pdfm {
        wm = false;
    }

    if matches.is_present("in") && matches.is_present("out") {
        let input = matches.value_of("in").unwrap();
        let output = matches.value_of("out").unwrap();
        let p = Path::new(input).canonicalize();
        if p.is_ok() {
            let pa = p.unwrap();
            println!("Canonicalized {:#?}", pa);
            let metadata = metadata(pa);
            if metadata.is_ok() {
                let md = metadata.unwrap();
                if md.is_dir() {
                    println!("Discovering Files...");
                    let paths = read_directory(input);
                    let tot = paths.len();
                    println!("Discovered {} Files", tot);
                    println!("Migrating incompatible files...");
                    let processes = handle_non_md(paths, input, output);
                    let ptot = processes.len();
                    let mut index = tot - ptot;
                    println!("Migrated {} Files", index);

                    for (source, destination) in processes {
                        read_file(source.clone(), destination.clone(), wm, false, false, &mut pdf_app);
                        if pdf || pdfm {
                            read_file(source, destination, wm, pdf, pdfm, &mut pdf_app)
                        }

                        index += 1;
                    }
                    println!("\rCompiled {} Files.        ", ptot);
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
                    read_file(pathstr.to_string(), final_path, wm, pdf, pdfm, &mut pdf_app);
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

fn read_file(path: String, out_path: String, wm: bool, pdf: bool, pdfm: bool, pdf_app: &mut PdfApplication) {
    // Read the file
    let mut content_file = File::open(&path).unwrap();
    let mut content = String::new();
    content_file.read_to_string(&mut content).unwrap();

    // Read the theme's stylesheet
    let mut css: String = String::new();
    let mut res = File::open("./dark.css").unwrap();
    let _read = res.read_to_string(&mut css).unwrap();

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

    let matcher = Regex::new(r"<\s*a[^>]*>").unwrap();
    let htclone = html.clone();
    let matches = matcher.find_iter(&htclone).collect::<Vec<regex::Match>>();
    for token in matches {
        let content = token.as_str();
        let fragment = Html::parse_fragment(content);
        let body = html.split(content).collect::<Vec<&str>>();
        let slice = fragment.select(&Selector::parse(r#"a"#).unwrap()).next().unwrap();
        let link = slice.value().attr("href");

        if link.is_some() {
            let mut href: String = String::from(link.unwrap());
            let reg = Regex::new(r"\S[^\s]*:/*[^\s]*|\S[^\s]*:|//[^\s]").unwrap();
            if reg.is_match(href.as_str()) {
                // ignore the link
            } else {
                if !href.contains('#') {
                    if pdf || pdfm {
                        href = format!("{}.pdf", href);
                    } else {
                        href = format!("{}.html", href);
                    }
                } else {
                    let mut sref: String = String::new();
                    let collection = href.split('#').collect::<Vec<&str>>();
                    if pdf || pdfm {
                        sref = collection.join(".pdf");
                    } else {
                        sref = collection.join(".html#");
                    }
                    href = sref;
                }
                html = body.join(format!("<a href=\"{}\">", href).as_str())
            }
        } else {
            println!("\rWARNING: {} contains an empty link", path)
        }
    }
    let mut out: File;
    let meta = metadata(&out_path);
    if meta.is_ok() {
        let trydel = remove_file(&out_path);
        if trydel.is_ok() {
            out = File::create(&out_path.replace(".md", ".html").replace(".markdown", ".html")).unwrap();
            write(html, &mut out)
        } else {
            println!("Failed to get metadata for \"{}\".\nReason: {:#?}", out_path, trydel.unwrap_err());
        }
    } else {
        let op = out_path.clone();
        let mut entries = op.split("/").collect::<Vec<&str>>();
        let name = entries.pop().unwrap();
        let dir_path = entries.join("/");
        create_dir_all(dir_path);
        if pdf {
            println!("Compiled: {} (PDF)", &out_path.replace(".md", ".pdf").replace(".markdown", ".pdf"));
        } else if pdfm {
            println!("Compiled: {} (PDF)", &out_path.replace(".md", ".pdf").replace(".markdown", ".pdf"));
            let mut pdf_content = pdf_app.builder()
                .orientation(Orientation::Landscape)
                .title(name)
                .margin(Size::Millimeters(0))
                .build_from_html(&html)
                .expect("failed to build pdf");

            pdf_content.save(&out_path.replace(".md", ".pdf").replace(".markdown", ".pdf")).expect("failed to save foo.pdf");
        } else {
            println!("Compiled: {} (HTML)", &out_path.replace(".md", ".html").replace(".markdown", ".html"));
            out = File::create(&out_path.replace(".md", ".html").replace(".markdown", ".html")).unwrap();
            write(html, &mut out)
        }
    }
}

fn write(content: String, file: &mut File) {
    file.write(content.as_bytes());
}

fn parse(content: String) -> String {
    let parsed = markdown_to_html(&content.as_str(), &ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            tagfilter: false,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            header_ids: Some("".to_string()),
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
    });
    parsed
}