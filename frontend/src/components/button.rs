use seed::{prelude::*, *};
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Button<Msg> {
    text: &'static str,
    style: &'static str,
    icon: &'static str,

    on_click: Rc<dyn Fn() -> Msg>,
    marker: PhantomData<Msg>,
}

impl<Msg: 'static> Button<Msg> {
    pub fn create(on_click: impl Fn() -> Msg + Clone + 'static) -> Self {
        Self::new("create", "is-success", "fa-plus", on_click)
    }

    pub fn confirm(on_click: impl Fn() -> Msg + Clone + 'static) -> Self {
        Self::new("confirm", "is-success", "fa-check", on_click)
    }

    pub fn delete(on_click: impl Fn() -> Msg + Clone + 'static) -> Self {
        Self::new("delete", "is-danger", "fa-times", on_click)
    }

    pub fn trash(on_click: impl Fn() -> Msg + Clone + 'static) -> Self {
        Self::new("", "is-danger", "fa-trash", on_click)
    }

    pub fn save(on_click: impl Fn() -> Msg + Clone + 'static) -> Self {
        Self::new("", "is-success", "fa-save", on_click)
    }

    pub fn new(
        text: &'static str,
        style: &'static str,
        icon: &'static str,
        on_click: impl Fn() -> Msg + Clone + 'static,
    ) -> Self {
        Button {
            text,
            style,
            icon,
            on_click: Rc::new(on_click),
            marker: PhantomData::default(),
        }
    }

    pub fn view(&self, disabled: bool) -> Node<Msg> {
        let func = self.on_click.clone();

        button![
            C!["button", self.style],
            ev(Ev::Click, move |_| func()),
            attrs! {At::Disabled => disabled.as_at_value()},
            span![C!["icon", "is-small"], i![C!["fas", self.icon]]],
            IF!(!self.text.is_empty() => span![self.text])
        ]
    }
}
