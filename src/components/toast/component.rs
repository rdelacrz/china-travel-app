use dioxus::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[css_module("/src/components/toast/style.css")]
struct Styles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToastOptions {
    description: Option<String>,
    duration: Option<Duration>,
    permanent: bool,
}

impl ToastOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn permanent(mut self, permanent: bool) -> Self {
        self.permanent = permanent;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToastRecord {
    id: usize,
    title: String,
    description: Option<String>,
    toast_type: ToastType,
}

#[derive(Clone, Copy)]
pub struct ToastApi {
    push: Callback<(String, ToastType, ToastOptions)>,
}

impl ToastApi {
    pub fn success(&self, title: String, options: ToastOptions) {
        self.push.call((title, ToastType::Success, options));
    }

    pub fn error(&self, title: String, options: ToastOptions) {
        self.push.call((title, ToastType::Error, options));
    }

    pub fn warning(&self, title: String, options: ToastOptions) {
        self.push.call((title, ToastType::Warning, options));
    }

    pub fn info(&self, title: String, options: ToastOptions) {
        self.push.call((title, ToastType::Info, options));
    }
}

pub fn use_toast() -> ToastApi {
    use_context()
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    const DEFAULT_DURATION: Duration = Duration::from_secs(5);
    const MAX_TOASTS: usize = 10;
    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    let mut toasts = use_signal(VecDeque::<ToastRecord>::new);
    let push = use_callback(
        move |(title, toast_type, options): (String, ToastType, ToastOptions)| {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let record = ToastRecord {
                id,
                title,
                description: options.description,
                toast_type,
            };
            {
                let mut records = toasts.write();
                records.push_back(record);
                while records.len() > MAX_TOASTS {
                    records.pop_front();
                }
            }

            if !options.permanent {
                let duration = options.duration.unwrap_or(DEFAULT_DURATION);
                spawn(async move {
                    tokio::time::sleep(duration).await;
                    toasts.write().retain(|toast| toast.id != id);
                });
            }
        },
    );
    use_context_provider(|| ToastApi { push });

    let records = toasts.read().iter().cloned().collect::<Vec<_>>();
    let count = records.len();

    rsx! {
        {children}
        div {
            class: Styles::dx_toast_container,
            role: "region",
            aria_label: "{count} notifications",
            tabindex: "-1",
            style: "--toast-count: {count}",
            ol {
                for (index, toast) in records.into_iter().rev().enumerate() {
                    li { key: "toast-{toast.id}",
                        article {
                            class: Styles::dx_toast,
                            "data-type": toast.toast_type.as_str(),
                            "data-top": if index == 0 { "true" },
                            "data-toast-even": if index % 2 == 0 { "true" },
                            "data-toast-odd": if index % 2 == 1 { "true" },
                            style: "--toast-index: {index}; --toast-padding: 0;",
                            div { class: Styles::dx_toast_content,
                                p { class: Styles::dx_toast_title, "{toast.title}" }
                                if let Some(description) = toast.description {
                                    p { class: Styles::dx_toast_description, "{description}" }
                                }
                            }
                            button {
                                class: Styles::dx_toast_close,
                                r#type: "button",
                                aria_label: "Dismiss notification",
                                onpointerup: move |_| {
                                    toasts.write().retain(|record| record.id != toast.id);
                                },
                                "×"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn TriggerToast() -> Element {
        let toast = use_toast();
        use_hook(move || {
            toast.success(
                "Saved".to_string(),
                ToastOptions::new()
                    .description("Everything synced")
                    .permanent(true),
            );
        });
        rsx! {}
    }

    #[test]
    fn provider_renders_a_styled_notification_without_browser_shortcuts() {
        let mut dom = VirtualDom::new(|| rsx! { ToastProvider { TriggerToast {} } });
        dom.rebuild_in_place();
        dom.mark_all_dirty();
        dom.render_immediate_to_vec();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("Saved"));
        assert!(html.contains("Everything synced"));
        assert!(html.contains('×') || html.contains("&#215;") || html.contains("&times;"));
    }
}
