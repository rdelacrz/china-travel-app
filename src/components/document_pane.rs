use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::linked_text::LinkedText;
use crate::domain::TravelDocument;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronDown, ChevronUp, ExternalLink, Paperclip, Pencil};

#[component]
pub fn DocumentPane(
    document: TravelDocument,
    expanded: bool,
    busy: bool,
    on_toggle_view: EventHandler<MouseEvent>,
    on_edit: EventHandler<MouseEvent>,
    on_open_file: EventHandler<MouseEvent>,
    on_open_url: EventHandler<String>,
) -> Element {
    let collapsed_description = truncate(&document.description, 180);
    let attachment_label = document
        .attachment
        .as_ref()
        .and_then(|attachment| attachment.display_name.clone())
        .unwrap_or_else(|| "Attached file".to_string());
    rsx! {
        article { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
            div { class: "flex items-start gap-3",
                div { class: "min-w-0 flex-1",
                    h2 { class: "break-words text-lg font-semibold text-slate-950", "{document.name}" }
                    if expanded {
                        if document.description.is_empty() {
                            p { class: "mt-2 text-sm italic text-slate-500", "No description" }
                        } else {
                            LinkedText {
                                text: document.description.clone(),
                                on_open_url: move |url| on_open_url.call(url),
                            }
                        }
                    } else if collapsed_description.is_empty() {
                        p { class: "mt-2 text-sm italic text-slate-500", "No description" }
                    } else {
                        p { class: "mt-2 whitespace-pre-wrap break-words text-sm leading-6 text-slate-600", "{collapsed_description}" }
                    }
                    if document.attachment.is_some() {
                        p { class: "mt-3 flex items-center gap-1 truncate text-xs font-medium text-slate-500",
                            Paperclip { size: 14 }
                            span { class: "truncate", "{attachment_label}" }
                        }
                    }
                }
                div { class: "flex shrink-0 items-start gap-1",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        aria_label: if expanded { "Collapse document description" } else { "View full document description" },
                        title: if expanded { "Collapse description" } else { "View full description" },
                        disabled: busy,
                        onclick: move |event| on_toggle_view.call(event),
                        if expanded { ChevronUp { size: 18 } } else { ChevronDown { size: 18 } }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        aria_label: "Edit document",
                        title: "Edit document",
                        disabled: busy,
                        onclick: move |event| on_edit.call(event),
                        Pencil { size: 18 }
                    }
                    if document.attachment.is_some() {
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Icon,
                            aria_label: "Open attached file",
                            title: "Open attached file",
                            disabled: busy,
                            onclick: move |event| on_open_file.call(event),
                            ExternalLink { size: 18 }
                        }
                    }
                }
            }
        }
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::truncate;

    #[test]
    fn truncation_is_unicode_safe() {
        assert_eq!(truncate("你好世界", 3), "你好世…");
        assert_eq!(truncate("short", 10), "short");
    }
}
