use comrak::{markdown_to_html, ComrakOptions, ComrakExtensionOptions, ComrakParseOptions, ComrakRenderOptions};
use regex::Regex;
use scraper::{Html, Selector};

pub(crate) fn parse(content: String) -> String {
    markdown_to_html(&content.as_str(), &ComrakOptions {
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
    })
}

pub(crate) fn relink(h: String, replacement: &str) -> String {
    let mut html = h;
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
                if href.ends_with(".md") || href.ends_with(".markdown") {
                    href = href.replace(".md", "").replace(".markdown", "");
                    href = format!("{}.{}", href, replacement);
                } else if href.contains('#') {
                    let mut sref: String = String::new();
                    let collection = href.split('#').collect::<Vec<&str>>();
                    sref = collection.join(&*format!(".{}#", replacement));
                    href = sref;
                }
                html = body.join(format!("<a href=\"{}\">", href).as_str())
            }
        }
    };
    return html;
}

pub(crate) fn parse_theme(theme: &str) -> &str {
    return match theme.to_lowercase().as_str() {
        "noir" => "blockquote,h1,h2,h3,h4,h5,h6,p{margin:0;padding:0}body{font-family:\"Helvetica Neue\",Helvetica,Arial,sans-serif;font-size:13px;line-height:18px;color:#fff;background-color:#110f14;margin:10px 13px 10px 13px}table{margin:10px 0 15px 0;border-collapse:collapse}td,th{border:1px solid #ddd;padding:3px 10px}th{padding:5px 10px}a{color:#59acf3}a:hover{color:#a7d8ff;text-decoration:none}a img{border:none}p{margin-bottom:9px}h1,h2,h3,h4,h5,h6{color:#fff;line-height:36px}h1{margin-bottom:18px;font-size:30px}h2{font-size:24px}h3{font-size:18px}h4{font-size:16px}h5{font-size:14px}h6{font-size:13px}hr{margin:0 0 19px;border:0;border-bottom:1px solid #ccc}blockquote{padding:13px 13px 21px 15px;margin-bottom:18px;font-family:georgia,serif;font-style:italic}blockquote:before{content:\"\\201C\";font-size:40px;margin-left:-10px;font-family:georgia,serif;color:#eee}blockquote p{font-size:14px;font-weight:300;line-height:18px;margin-bottom:0;font-style:italic}code,pre{font-family:Menlo,Monaco,Andale Mono,Courier New,monospace}code{padding:1px 3px;font-size:12px;-webkit-border-radius:3px;-moz-border-radius:3px;border-radius:3px;background:#334}pre{display:block;padding:14px;margin:0 0 18px;line-height:16px;font-size:11px;border:1px solid #334;white-space:pre;white-space:pre-wrap;word-wrap:break-word;background-color:#282a36;border-radius:6px}pre code{font-size:11px;padding:0;background:0 0}sup{font-size:.83em;vertical-align:super;line-height:0}*{-webkit-print-color-adjust:exact}@media screen and (min-width:914px){body{width:854px;margin:10px auto}}@media print{body,code,h1,h2,h3,h4,h5,h6,pre code{color:#000}pre,table{page-break-inside:avoid}}",
        "iridium" => "html{background-color:#282a36;scroll-behavior:smooth;width:100%;height:100%}.container{margin:2em 4em}body{padding:20px;color:#fff;font-size:15px;font-family:\"Lucida Grande\",\"Lucida Sans Unicode\",\"Lucida Sans\",AppleSDGothicNeo-Medium,'Segoe UI','Malgun Gothic',Verdana,Tahoma,sans-serif;-webkit-font-smoothing:antialiased}a{color:#b97ec4}a:hover{color:#8bb156}h2{border-bottom:1px solid #21232d;line-height:1.7em}h6{color:#a4a296}hr{border:1px solid #21232d}pre>code{font-size:.9em;font-family:Consolas,Inconsolata,Courier,monospace}blockquote{border-left:4px solid #121319;padding:0 15px;font-style:italic}img{max-width:100%}table{width:100%;margin:0;border:.2em solid #4b4e65;border-radius:.4em;background-color:#333442}table tr td,table tr th{padding:1em}table tr:nth-child(2n){background-color:#494d63}.hljs-comment,.hljs-quote{color:#8d8687}.hljs-link,.hljs-meta,.hljs-name,.hljs-regexp,.hljs-selector-class,.hljs-selector-id,.hljs-tag,.hljs-template-variable,.hljs-variable{color:#ef6155}.hljs-built_in,.hljs-builtin-name,.hljs-deletion,.hljs-literal,.hljs-number,.hljs-params,.hljs-type{color:#f99b15}.hljs-attribute,.hljs-section,.hljs-title{color:#fec418}.hljs-addition,.hljs-bullet,.hljs-string,.hljs-symbol{color:#48b685}.hljs-keyword,.hljs-selector-tag{color:#815ba4}.hljs{display:block;overflow-x:auto;background:#1e2029;color:#a39e9b;padding:.5em;border-radius:.2em}.hljs-emphasis{font-style:italic}.hljs-strong{font-weight:700}",
        "iridium-light" => "html{background-color:#f1f1f1;scroll-behavior:smooth;width:100%;height:100%}.container{margin:2em 4em}body{padding:20px;color:#000;font-size:15px;font-family:\"Lucida Grande\",\"Lucida Sans Unicode\",\"Lucida Sans\",AppleSDGothicNeo-Medium,'Segoe UI','Malgun Gothic',Verdana,Tahoma,sans-serif;-webkit-font-smoothing:antialiased}a{color:#ff008c}a:hover{color:#62cd28}h2{border-bottom:1px solid #21232d;line-height:1.7em}h6{color:#a4a296}hr{border:1px solid #21232d}pre>code{font-size:.9em;font-family:Consolas,Inconsolata,Courier,monospace}blockquote{border-left:4px solid #121319;padding:0 15px;font-style:italic}img{max-width:100%}table{width:100%;margin:0;border-radius:.4em;background-color:#adaeb8}table tr td,table tr th{padding:1em}table tr th{background-color:#adaeb8}table tr:nth-child(2n){background-color:#999}.hljs-comment,.hljs-quote{color:#8d8687}.hljs-link,.hljs-meta,.hljs-name,.hljs-regexp,.hljs-selector-class,.hljs-selector-id,.hljs-tag,.hljs-template-variable,.hljs-variable{color:#ef6155}.hljs-built_in,.hljs-builtin-name,.hljs-deletion,.hljs-literal,.hljs-number,.hljs-params,.hljs-type{color:#f99b15}.hljs-attribute,.hljs-section,.hljs-title{color:#fec418}.hljs-addition,.hljs-bullet,.hljs-string,.hljs-symbol{color:#48b685}.hljs-keyword,.hljs-selector-tag{color:#815ba4}.hljs{display:block;overflow-x:auto;background:#1e2029;color:#a39e9b;padding:.5em;border-radius:.2em}.hljs-emphasis{font-style:italic}.hljs-strong{font-weight:700}",
        _ => "html{background-color:#282a36;scroll-behavior:smooth;width:100%;height:100%}.container{margin:2em 4em}body{padding:20px;color:#fff;font-size:15px;font-family:\"Lucida Grande\",\"Lucida Sans Unicode\",\"Lucida Sans\",AppleSDGothicNeo-Medium,'Segoe UI','Malgun Gothic',Verdana,Tahoma,sans-serif;-webkit-font-smoothing:antialiased}a{color:#b97ec4}a:hover{color:#8bb156}h2{border-bottom:1px solid #21232d;line-height:1.7em}h6{color:#a4a296}hr{border:1px solid #21232d}pre>code{font-size:.9em;font-family:Consolas,Inconsolata,Courier,monospace}blockquote{border-left:4px solid #121319;padding:0 15px;font-style:italic}img{max-width:100%}table{width:100%;margin:0;border:.2em solid #4b4e65;border-radius:.4em;background-color:#333442}table tr td,table tr th{padding:1em}table tr:nth-child(2n){background-color:#494d63}.hljs-comment,.hljs-quote{color:#8d8687}.hljs-link,.hljs-meta,.hljs-name,.hljs-regexp,.hljs-selector-class,.hljs-selector-id,.hljs-tag,.hljs-template-variable,.hljs-variable{color:#ef6155}.hljs-built_in,.hljs-builtin-name,.hljs-deletion,.hljs-literal,.hljs-number,.hljs-params,.hljs-type{color:#f99b15}.hljs-attribute,.hljs-section,.hljs-title{color:#fec418}.hljs-addition,.hljs-bullet,.hljs-string,.hljs-symbol{color:#48b685}.hljs-keyword,.hljs-selector-tag{color:#815ba4}.hljs{display:block;overflow-x:auto;background:#1e2029;color:#a39e9b;padding:.5em;border-radius:.2em}.hljs-emphasis{font-style:italic}.hljs-strong{font-weight:700}"
    };
}