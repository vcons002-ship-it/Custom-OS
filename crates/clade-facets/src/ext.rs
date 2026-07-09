//! Extension → facet(s). A weaker signal than magic bytes (extensions lie),
//! so lower confidence. Some extensions yield multiple facets (an `.svg` is
//! both an image and XML/code).

use crate::Facet;

/// Facets implied by a lowercased extension (no dot). Empty if unknown.
pub fn from_extension(ext: &str) -> Vec<Facet> {
    let f = |kind: &str, conf: f32, detail: Option<&str>| Facet {
        kind: kind.into(),
        confidence: conf,
        detail: detail.map(str::to_string),
    };
    match ext {
        // Images.
        "jpg" | "jpeg" => vec![f("image", 0.85, Some("jpeg"))],
        "png" => vec![f("image", 0.85, Some("png"))],
        "gif" => vec![f("image", 0.85, Some("gif"))],
        "bmp" => vec![f("image", 0.85, Some("bmp"))],
        "tif" | "tiff" => vec![f("image", 0.85, Some("tiff"))],
        "webp" => vec![f("image", 0.85, Some("webp"))],
        "heic" | "heif" => vec![f("image", 0.85, Some("heic"))],
        "svg" => vec![f("image", 0.8, Some("svg")), f("code", 0.6, Some("xml"))],
        // Documents.
        "pdf" => vec![f("pdf", 0.85, Some("pdf"))],
        "docx" | "odt" | "rtf" | "doc" => vec![f("document", 0.85, Some(ext))],
        // Data.
        "json" => vec![f("data", 0.85, Some("json"))],
        "ndjson" | "jsonl" => vec![f("data", 0.85, Some("ndjson"))],
        "csv" => vec![f("data", 0.85, Some("csv"))],
        "tsv" => vec![f("data", 0.85, Some("tsv"))],
        "yaml" | "yml" => vec![f("data", 0.8, Some("yaml"))],
        "toml" => vec![f("data", 0.8, Some("toml"))],
        "xml" => vec![f("data", 0.7, Some("xml")), f("code", 0.5, Some("xml"))],
        // Markup / notes.
        "md" | "markdown" => vec![f("markdown", 0.85, None), f("text", 0.7, None)],
        "txt" | "text" | "log" => vec![f("text", 0.85, None)],
        // Code — extension → language.
        _ => code_from_ext(ext)
            .map(|lang| vec![f("code", 0.85, Some(lang))])
            .unwrap_or_default(),
    }
}

/// Map an extension to a source-code language, if it is one.
pub fn code_from_ext(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "rs" => "rust",
        "py" | "pyw" => "python",
        "js" | "mjs" | "cjs" => "javascript",
        "ts" => "typescript",
        "tsx" => "tsx",
        "jsx" => "jsx",
        "c" | "h" => "c",
        "cc" | "cpp" | "cxx" | "hpp" | "hh" => "cpp",
        "go" => "go",
        "java" => "java",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "sh" | "bash" | "zsh" => "shell",
        "pl" | "pm" => "perl",
        "lua" => "lua",
        "sql" => "sql",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "cs" => "csharp",
        "fs" => "fsharp",
        "scala" => "scala",
        "clj" | "cljs" => "clojure",
        "ex" | "exs" => "elixir",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "jl" => "julia",
        "nim" => "nim",
        "zig" => "zig",
        "dart" => "dart",
        "r" => "r",
        "vue" => "vue",
        "vim" => "vimscript",
        _ => return None,
    })
}
