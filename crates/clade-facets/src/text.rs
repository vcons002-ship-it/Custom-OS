//! Text-family content heuristics: is this bytes-as-text, and if so is it
//! code, notes, a to-do list, markdown, or structured data? Signals from
//! content, blended with (not overridden by) the extension's guess.

use crate::ext;
use crate::Facet;

/// True if the bytes look like human-readable UTF-8 text (no NULs, few control
/// characters). Samples the head so a huge file is cheap.
pub fn is_text(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let sample = &bytes[..bytes.len().min(8192)];
    // A NUL almost always means binary.
    if sample.contains(&0) {
        return false;
    }
    // Trim to a UTF-8 boundary so a multibyte char split at the sample edge
    // doesn't cause a false negative.
    let text = match std::str::from_utf8(sample) {
        Ok(s) => s,
        Err(e) => match std::str::from_utf8(&sample[..e.valid_up_to()]) {
            Ok(s) if !s.is_empty() => s,
            _ => return false,
        },
    };
    let control = text
        .chars()
        .filter(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
        .count();
    (control as f32 / text.chars().count().max(1) as f32) < 0.02
}

/// Facets derived from text content + the extension's language hint.
pub fn text_facets(ext: &str, text: &str) -> Vec<Facet> {
    let mut out = Vec::new();
    let head: String = text.chars().take(8192).collect();

    // Code: extension language wins; else infer from a shebang or markers.
    let lang = ext::code_from_ext(ext);
    if let Some(lang) = lang {
        out.push(facet("code", 0.9, Some(lang)));
    } else if let Some(l) = shebang_lang(&head) {
        out.push(facet("code", 0.85, Some(l)));
    } else if looks_like_code(&head) {
        out.push(facet("code", 0.65, None));
    }

    // Markdown.
    if matches!(ext, "md" | "markdown") || looks_like_markdown(&head) {
        out.push(facet("markdown", if ext == "md" { 0.9 } else { 0.6 }, None));
    }

    // Structured data.
    if matches!(ext, "json" | "ndjson" | "jsonl") || looks_like_json(&head) {
        out.push(facet("data", 0.8, Some("json")));
    } else if matches!(ext, "csv" | "tsv") || looks_like_csv(&head) {
        out.push(facet("data", 0.7, Some("csv")));
    } else if matches!(ext, "yaml" | "yml" | "toml") {
        out.push(facet("data", 0.8, Some(ext)));
    }

    // A to-do list (checkbox lines) — additive with text/notes.
    if has_checkboxes(&head) {
        out.push(facet("todo", 0.8, None));
    }

    // Everything readable is at least text; prose-ish text is also "notes".
    out.push(facet("text", 0.6, None));
    if out.iter().all(|f| f.kind != "code" && f.kind != "data") && looks_like_prose(&head) {
        out.push(facet("notes", 0.6, None));
    }
    out
}

fn shebang_lang(text: &str) -> Option<&'static str> {
    let first = text.lines().next()?;
    if !first.starts_with("#!") {
        return None;
    }
    let l = first.to_ascii_lowercase();
    Some(if l.contains("python") {
        "python"
    } else if l.contains("bash") || l.contains("/sh") || l.contains("zsh") {
        "shell"
    } else if l.contains("node") {
        "javascript"
    } else if l.contains("ruby") {
        "ruby"
    } else if l.contains("perl") {
        "perl"
    } else {
        "shell"
    })
}

fn looks_like_code(text: &str) -> bool {
    const MARKERS: &[&str] = &[
        "fn ",
        "def ",
        "function ",
        "class ",
        "import ",
        "#include",
        "package ",
        "public ",
        "private ",
        "const ",
        "let ",
        "var ",
        "return ",
        "=>",
        "func ",
    ];
    let hits = MARKERS.iter().filter(|m| text.contains(**m)).count();
    let punct = text.matches(['{', '}', ';']).count();
    hits >= 2 && punct >= 3
}

fn looks_like_markdown(text: &str) -> bool {
    let heading = text
        .lines()
        .any(|l| l.starts_with("# ") || l.starts_with("## "));
    let list = text
        .lines()
        .filter(|l| l.trim_start().starts_with("- "))
        .count()
        >= 2;
    let link = text.contains("](");
    heading && (list || link)
}

fn looks_like_json(text: &str) -> bool {
    let t = text.trim_start();
    (t.starts_with('{') && t.contains("\":")) || (t.starts_with('[') && t.contains('{'))
}

fn looks_like_csv(text: &str) -> bool {
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(5)
        .collect();
    if lines.len() < 2 {
        return false;
    }
    let commas = |l: &str| l.matches(',').count();
    let c0 = commas(lines[0]);
    c0 >= 1 && lines.iter().all(|l| commas(l) == c0)
}

fn has_checkboxes(text: &str) -> bool {
    text.lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("- [ ]")
                || t.starts_with("- [x]")
                || t.starts_with("* [ ]")
                || t.starts_with("* [x]")
                || t.starts_with("[ ]")
                || t.starts_with("[x]")
        })
        .count()
        >= 1
}

fn looks_like_prose(text: &str) -> bool {
    // Sentences with spaces and terminal punctuation, not dominated by symbols.
    let words = text.split_whitespace().count();
    let sentences = text.matches(['.', '!', '?']).count();
    words >= 8 && sentences >= 1
}

fn facet(kind: &str, conf: f32, detail: Option<&str>) -> Facet {
    Facet {
        kind: kind.into(),
        confidence: conf,
        detail: detail.map(str::to_string),
    }
}
