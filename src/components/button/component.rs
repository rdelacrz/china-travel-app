use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[css_module("/src/components/button/style.css")]
struct Styles;

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Destructive => "destructive",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Link => "link",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonSize {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

impl ButtonSize {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonSize::Xs => "xs",
            ButtonSize::Sm => "sm",
            ButtonSize::Default => "default",
            ButtonSize::Lg => "lg",
            ButtonSize::Icon => "icon",
            ButtonSize::IconXs => "icon-xs",
            ButtonSize::IconSm => "icon-sm",
            ButtonSize::IconLg => "icon-lg",
        }
    }
}

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    #[props(extends=GlobalAttributes)]
    #[props(extends=button)]
    attributes: Vec<Attribute>,
    on_press: Option<EventHandler<()>>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    children: Element,
) -> Element {
    let base = attributes!(button {
        class: Styles::dx_button,
        "data-style": variant.class(),
        "data-size": size.class(),
    });
    let forwarded_attributes = attributes
        .into_iter()
        .filter(|attribute| !matches!(attribute.name, "onclick" | "onpointerup" | "onkeydown"))
        .collect();
    // Dioxus 0.7 LiveView omits `click` listeners when attributes are merged dynamically.
    // `pointerup` is emitted for touch, pen, and mouse input, so it is the activation transport.
    let pointer_handler = on_press;
    let keyboard_handler = on_press;
    let event_attributes = attributes!(button {
        onpointerup: move |_| {
            if let Some(handler) = &pointer_handler {
                handler.call(());
            }
        },
        onkeydown: move |event| {
            let key = event.key();
            if key == Key::Enter || key == Key::Character(" ".to_string()) {
                if let Some(handler) = &keyboard_handler {
                    event.prevent_default();
                    handler.call(());
                }
            }
            if let Some(handler) = &onkeydown {
                handler.call(event);
            }
        },
    });
    let merged = merge_attributes(vec![base, forwarded_attributes, event_attributes]);

    rsx! {
        button {
            ..merged,
            {children}
        }
    }
}
