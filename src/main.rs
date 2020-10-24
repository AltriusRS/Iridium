use comrak::{markdown_to_html, ComrakOptions};
const HEADERS: &str = "<link rel=\"stylesheet\" href=\"http://github.com/yrgoldteeth/darkdowncss/raw/master/darkdown.css\" />\n<link rel=\"stylesheet\" href=\"//cdnjs.cloudflare.com/ajax/libs/highlight.js/10.1.2/styles/default.min.css\">\n<script src=\"//cdnjs.cloudflare.com/ajax/libs/highlight.js/10.1.2/highlight.min.js\"></script>\n<script>hljs.initHighlightingOnLoad();</script>";

fn main() {
    let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n<style>* {color: #f1f1f1; background-color: #111}.container {margin: 2pc 15% 2pc 15%}</style>");
    html = format!("{}\n{}\n</head>\n<body>\n<div class=\"container\">", html, HEADERS);
    html = format!("{}\n{}\n</div>\n</body>\n<html>", html, markdown_to_html("Hello, **World**!", &ComrakOptions::default()));
    println!("{}", html);
}
