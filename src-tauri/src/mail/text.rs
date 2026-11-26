//! A mail that is the document itself is kept as its prepared text (#31, the owner's word of
//! 2026-11-26): a head with sender, recipient, date, subject and attachments, then the text, in
//! lines as code and the model read best. A table row of an HTML mail is one line with its cells
//! side by side, so "Gesamt 23,80 €" stands together as on a printed invoice. Styles, scripts,
//! images, links and hidden padding are dropped, nothing of the mail is ever loaded.

use mail_parser::Message;

use crate::documents::MAIL_MARK;

/// Tags that end a line.
const BLOCKS: &[&str] = &[
    "p",
    "div",
    "br",
    "tr",
    "li",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "table",
    "tbody",
    "thead",
    "tfoot",
    "section",
    "article",
    "header",
    "footer",
    "blockquote",
    "hr",
    "ul",
    "ol",
    "center",
    "dt",
    "dd",
    "pre",
    "address",
    "main",
    "nav",
    "form",
];

/// Blocks that stand apart from the text around them by an empty line.
const PARAGRAPHS: &[&str] = &[
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "table",
    "blockquote",
    "ul",
    "ol",
    "hr",
];

/// Text of HTML as it shows: entities decoded, line ends and tabs of the source are spaces.
fn flat(text: &str) -> String {
    decode(&text.replace(['\r', '\n', '\t'], " "))
}

/// Ends the line, with an empty line after it when `ends` is 2. Never at the start.
fn break_line(out: &mut String, ends: usize) {
    while out.ends_with(' ') {
        out.pop();
    }
    if out.is_empty() {
        return;
    }
    let present = out.chars().rev().take_while(|c| *c == '\n').count();
    for _ in present..ends {
        out.push('\n');
    }
}

/// Tags whose whole content is dropped.
const DROPPED: &[&str] = &[
    "style", "script", "head", "title", "noscript", "template", "svg",
];

fn entity(name: &str) -> Option<char> {
    if let Some(number) = name.strip_prefix('#') {
        let code = match number.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => number.parse().ok()?,
        };
        return char::from_u32(code);
    }
    Some(match name {
        "nbsp" | "ensp" | "emsp" | "thinsp" => ' ',
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "euro" => '€',
        "auml" => 'ä',
        "ouml" => 'ö',
        "uuml" => 'ü',
        "Auml" => 'Ä',
        "Ouml" => 'Ö',
        "Uuml" => 'Ü',
        "szlig" => 'ß',
        "eacute" => 'é',
        "egrave" => 'è',
        "agrave" => 'à',
        "ndash" => '–',
        "mdash" => '—',
        "hellip" => '…',
        "bull" => '•',
        "middot" => '·',
        "copy" => '©',
        "reg" => '®',
        "trade" => '™',
        "laquo" => '«',
        "raquo" => '»',
        "bdquo" => '„',
        "ldquo" => '“',
        "rdquo" => '”',
        "lsquo" => '‘',
        "rsquo" => '’',
        "sbquo" => '‚',
        "times" => '×',
        "deg" => '°',
        "sect" => '§',
        "para" => '¶',
        "shy" | "zwnj" | "zwj" | "lrm" | "rlm" => '\u{200b}',
        _ => return None,
    })
}

/// Decodes the entities of a run of HTML text.
fn decode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let decoded = rest
            .find(';')
            .filter(|end| *end <= 10)
            .and_then(|end| entity(&rest[1..end]).map(|c| (c, end)));
        match decoded {
            Some((c, end)) => {
                out.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Characters a mail pads with to hide its preview, or that only steer the direction of text.
fn invisible(c: char) -> bool {
    matches!(
        c,
        '\u{200b}'..='\u{200f}' | '\u{00ad}' | '\u{034f}' | '\u{2060}'..='\u{2064}' | '\u{feff}' | '\u{202a}'..='\u{202e}'
    )
}

/// Whether a web address, which neither code nor the model needs.
fn is_address(word: &str) -> bool {
    let word = word.trim_matches(['<', '>', '(', ')', '[', ']', '"', '\'']);
    word.starts_with("http://") || word.starts_with("https://") || word.starts_with("www.")
}

/// One line as kept: invisible characters gone, web addresses gone, spaces collapsed.
fn tidy_line(line: &str) -> String {
    let visible: String = line
        .chars()
        .filter(|c| !invisible(*c))
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    visible
        .split(' ')
        .filter(|word| !word.is_empty() && !is_address(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Lines as kept: each tidied, no line of only dashes or stars, at most one empty line in a row,
/// none at the start or end.
fn tidy(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = tidy_line(line);
        let decoration = !line.is_empty() && line.chars().all(|c| "-_=*#~|.•·".contains(c));
        let line = if decoration { String::new() } else { line };
        if line.is_empty() && lines.last().is_none_or(String::is_empty) {
            continue;
        }
        lines.push(line);
    }
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines.join("\n")
}

/// The name of a tag, `td` of `<td class="x">` and of `</td>`, in lower case.
fn tag_name(tag: &str) -> String {
    tag.trim_start_matches('/')
        .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

/// Whether a tag hides its content (`display:none`, `hidden`), as a mail's preview padding does.
fn hides(tag: &str) -> bool {
    let tag = tag.to_ascii_lowercase();
    let packed = tag.replace(' ', "");
    packed.contains("display:none")
        || packed.contains("visibility:hidden")
        || tag.split_whitespace().skip(1).any(|word| word == "hidden")
}

/// The text of an HTML mail in lines: a line per block, a table row per line with its cells
/// side by side.
pub fn html_to_text(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    // The tag whose content is dropped, and how deep it nests in itself.
    let mut dropping: Option<(String, usize)> = None;
    while !rest.is_empty() {
        let Some(open) = rest.find('<') else {
            if dropping.is_none() {
                out.push_str(&flat(rest));
            }
            break;
        };
        if dropping.is_none() {
            out.push_str(&flat(&rest[..open]));
        }
        rest = &rest[open..];
        if let Some(comment) = rest.strip_prefix("<!--") {
            rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
            continue;
        }
        let Some(close) = rest.find('>') else {
            break;
        };
        let tag = &rest[1..close];
        rest = &rest[close + 1..];
        let name = tag_name(tag);
        let closing = tag.starts_with('/');
        let empty = tag.ends_with('/')
            || matches!(
                name.as_str(),
                "br" | "hr" | "img" | "meta" | "link" | "input"
            );
        if let Some((dropped, depth)) = &mut dropping {
            if name == *dropped && !empty {
                if closing {
                    *depth -= 1;
                    if *depth == 0 {
                        dropping = None;
                    }
                } else {
                    *depth += 1;
                }
            }
            continue;
        }
        if !closing && !empty && (DROPPED.contains(&name.as_str()) || hides(tag)) {
            dropping = Some((name, 1));
            continue;
        }
        if BLOCKS.contains(&name.as_str()) {
            let paragraph = PARAGRAPHS.contains(&name.as_str()) && (closing || name != "p");
            break_line(&mut out, if paragraph { 2 } else { 1 });
        } else if matches!(name.as_str(), "td" | "th") && !closing {
            out.push(' ');
        }
    }
    tidy(&out)
}

/// The text of a mail: its plain text where it has one, else its HTML in lines.
pub fn body_of(message: &Message<'_>) -> String {
    let html_only = message
        .text_body()
        .next()
        .is_none_or(|part| part.is_content_type("text", "html"));
    if html_only && let Some(html) = message.body_html(0) {
        return html_to_text(&html);
    }
    tidy(&message.body_text(0).unwrap_or_default())
}

/// The file of a mail: the mark, the head lines, an empty line, the text.
pub fn mail_file(head: &[String], body: &str) -> String {
    let mut file = String::from(MAIL_MARK);
    file.push('\n');
    for line in head {
        file.push_str(&tidy_line(line));
        file.push('\n');
    }
    file.push('\n');
    file.push_str(body);
    file.push('\n');
    file
}
