use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::linked_text::LinkedText;
use crate::domain::TravelDocument;
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronDown, ChevronUp, ExternalLink, Paperclip, Pencil, Trash2};

#[component]
pub fn DocumentPane(
    document: TravelDocument,
    expanded: bool,
    busy: bool,
    on_toggle_view: EventHandler<()>,
    on_edit: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_open_file: EventHandler<()>,
    on_open_url: EventHandler<String>,
) -> Element {
    let attachment_label = document
        .attachment
        .as_ref()
        .and_then(|attachment| attachment.display_name.clone())
        .unwrap_or_else(|| "Attached file".to_string());
    rsx! {
        article { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
            div { class: "flex items-center gap-3",
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Icon,
                    class: "shrink-0",
                    aria_label: if expanded { "Collapse document description" } else { "Expand document description" },
                    title: if expanded { "Collapse description" } else { "Expand description" },
                    disabled: busy,
                    on_press: move |_| on_toggle_view.call(()),
                    if expanded { ChevronUp { size: 20 } } else { ChevronDown { size: 20 } }
                }
                div { class: "min-w-0 flex-1",
                    h2 { class: "truncate text-lg font-semibold text-slate-950", "{document.name}" }
                    if expanded {
                        if document.description.is_empty() {
                            p { class: "mt-2 text-sm italic text-slate-500", "No description" }
                        } else {
                            LinkedText {
                                text: document.description.clone(),
                                on_open_url: move |url| on_open_url.call(url),
                            }
                        }
                    } else if document.description.is_empty() {
                        p { class: "mt-2 text-sm italic text-slate-500", "No description" }
                    } else {
                        p { class: "mt-2 truncate text-sm leading-6 text-slate-600", "{document.description}" }
                    }
                    if document.attachment.is_some() {
                        div { class: "mt-3 flex min-w-0 items-center gap-2 text-xs font-medium text-slate-500",
                            Paperclip { size: 14 }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconXs,
                                class: "min-w-0 flex-1 justify-start truncate px-1 text-left text-slate-500 hover:bg-slate-50 hover:text-slate-800 hover:underline",
                                aria_label: "Open attached file {attachment_label}",
                                title: "Open attached file",
                                disabled: busy,
                                on_press: move |_| on_open_file.call(()),
                                "{attachment_label}"
                            }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconXs,
                                class: "shrink-0 text-slate-500",
                                aria_label: "Open attached file",
                                title: "Open attached file",
                                disabled: busy,
                                on_press: move |_| on_open_file.call(()),
                                ExternalLink { size: 16 }
                            }
                        }
                    }
                }
                div { class: "flex shrink-0 items-center gap-1",
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        aria_label: "Edit document",
                        title: "Edit document",
                        disabled: busy,
                        on_press: move |_| on_edit.call(()),
                        Pencil { size: 18 }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        class: "text-slate-400 hover:bg-red-50 hover:text-red-700",
                        aria_label: "Delete document",
                        title: "Delete document",
                        disabled: busy,
                        on_press: move |_| on_delete.call(()),
                        Trash2 { size: 18 }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
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
