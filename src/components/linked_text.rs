use dioxus::prelude::*;
use linkify::{LinkFinder, LinkKind};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextSegment {
    text: String,
    url: Option<String>,
}

#[component]
pub fn LinkedText(text: String, on_open_url: EventHandler<String>) -> Element {
    let segments = link_segments(&text);
    rsx! {
        p { class: "mt-2 whitespace-pre-wrap break-words text-sm leading-6 text-slate-700",
            for segment in segments {
                if let Some(url) = segment.url {
                    a {
                        href: "#",
                        class: "font-medium text-red-700 underline decoration-red-300 underline-offset-2 hover:text-red-800 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                        onclick: move |event| {
                            event.prevent_default();
                            on_open_url.call(url.clone());
                        },
                        "{segment.text}"
                    }
                } else {
                    span { "{segment.text}" }
                }
            }
        }
    }
}

fn link_segments(text: &str) -> Vec<TextSegment> {
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);
    finder.url_must_have_scheme(false);
    finder
        .spans(text)
        .map(|span| {
            let raw = span.as_str().to_string();
            let url = span.kind().and_then(|kind| {
                if *kind != LinkKind::Url {
                    return None;
                }
                let candidate = if raw.starts_with("www.") {
                    format!("https://{raw}")
                } else {
                    raw.clone()
                };
                let parsed = Url::parse(&candidate).ok()?;
                matches!(parsed.scheme(), "http" | "https").then(|| parsed.to_string())
            });
            TextSegment { text: raw, url }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::link_segments;

    #[test]
    fn segments_only_link_http_urls_and_preserve_text() {
        let segments =
            link_segments("Visit https://example.test/a, www.example.org/docs and javascript:bad");
        assert_eq!(segments[1].url.as_deref(), Some("https://example.test/a"));
        assert!(segments
            .iter()
            .any(|segment| segment.text == "www.example.org/docs" && segment.url.is_some()));
        assert!(segments
            .iter()
            .any(|segment| segment.text.contains("javascript:bad") && segment.url.is_none()));
    }

    #[test]
    fn multiline_text_is_not_collapsed() {
        let segments = link_segments("one\ntwo https://example.test\nthree");
        assert!(segments.iter().any(|segment| segment.text.contains('\n')));
    }
}
