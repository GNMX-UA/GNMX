// use seed::{prelude::*, *};
// use seed::prelude::web_sys::HtmlInputElement;
// use std::future::Future;
// use std::pin::Pin;
// use std::rc::Rc;
//
// use crate::fields::{Field, Suggestion};
//
// type Func = dyn Fn(String) -> Pin<Box<dyn Future<Output=Vec<Suggestion>>>>;
//
// #[derive(Clone, Debug)]
// pub enum Msg {
//     InputFocus,
//     InputBlur,
//     InputKeyDown(web_sys::KeyboardEvent),
//     InputClick(web_sys::MouseEvent),
//     InputChange(String),
//     SuggestionClick(usize),
//     SetIgnoreSuggestionBlur(bool),
//     Fetched(Vec<Suggestion>),
// }
//
// pub struct AutocompleteField
// {
//     completer: Rc<Func>,
//
//     suggestions: Vec<Suggestion>,
//     selected: Option<usize>,
//     value: Option<i64>,
//
//     is_open: bool,
//     is_ok: bool,
//     ignore_blur: bool,
//
//     elem: ElRef<HtmlInputElement>,
// }
//
// impl Field for AutocompleteField
// {
//     type Msg = Msg;
//     type Value = i64;
//
//     fn set(&self, value: Option<&Self::Value>) {
//         todo!()
//     }
//
//     fn submit(&mut self) -> Option<Self::Value> {
//         self.value
//     }
//
//     fn update(&mut self, msg: Self::Msg, orders: &mut impl Orders<Self::Msg>) -> bool{
//         match msg
//         {
//             Msg::InputFocus => self.is_open = true,
//             Msg::InputBlur => self.is_open = self.ignore_blur,
//             Msg::InputKeyDown(event) => self.handle_key(event),
//             Msg::InputClick(_) => self.is_open = true,
//             Msg::InputChange(string) => self.handle_change(string, orders),
//             Msg::SuggestionClick(index) => self.set_suggested_value(index),
//             Msg::SetIgnoreSuggestionBlur(ignore) => self.ignore_blur = ignore,
//             Msg::Fetched(suggestions) => self.suggestions = suggestions,
//         }
//
//         // TODO: this is wrong
//         true
//     }
//
//     fn view(&self) -> Vec<Node<Self::Msg>> {
//         vec![div![C!["field"],
//             label![C!["label"], "Name"],
//             div![C!["control", "has-icons-right"],
//                 self.view_dropdown()
//             ]
//         ]]
//     }
// }
//
//
// impl AutocompleteField
// {
//     pub fn new<R: 'static>(completer: impl Fn(String) -> R + 'static) -> Self
//         where R: Future<Output=Vec<Suggestion>>
//     {
//         Self {
//             completer: Rc::new(move |query| Box::pin(completer(query))),
//             suggestions: vec![],
//             selected: None,
//             is_open: false,
//             is_ok: false,
//             ignore_blur: false,
//             value: None,
//             elem: Default::default(),
//         }
//     }
//
//     pub fn value(&self) -> Option<i64>
//     {
//         self.value
//     }
//
//     fn set_suggested_value(&mut self, index: usize)
//     {
//         if let Some(elem) = self.elem.get()
//         {
//             let Suggestion { name, value } = &self.suggestions[index];
//
//             elem.set_value(name);
//             self.value = Some(*value);
//
//             self.is_ok = true;
//             self.is_open = false;
//             self.selected = None;
//         }
//     }
//
//     fn handle_key(&mut self, event: web_sys::KeyboardEvent)
//     {
//         match event.key().as_str()
//         {
//             "ArrowDown" =>
//                 {
//                     event.prevent_default();
//                     self.selected = match (self.suggestions.len(), self.selected)
//                     {
//                         (0, _) => None,
//                         (_, None) => Some(0),
//                         (len, Some(index)) => Some((len - 1).min(index + 1))
//                     }
//                 }
//             "ArrowUp" =>
//                 {
//                     event.prevent_default();
//                     self.selected = match (self.suggestions.len(), self.selected)
//                     {
//                         (0, _) => None,
//                         (_, None) => Some(self.suggestions.len() - 1),
//                         (_, Some(index)) => Some(index.max(1) - 1)
//                     }
//                 }
//             "Enter" =>
//                 {
//                     event.prevent_default();
//                     if let (Some(index), true) = (self.selected, self.is_open)
//                     {
//                         self.set_suggested_value(index)
//                     }
//                 }
//             "Escape" =>
//                 {
//                     self.is_open = false;
//                     self.selected = None;
//                 }
//             "Tab" => (),
//             _ => self.is_open = true
//         }
//     }
//
//     fn handle_change(&mut self, string: String, orders: &mut impl Orders<Msg>)
//     {
//         self.is_ok = false;
//
//         if string.is_empty()
//         {
//             self.is_open = false;
//             return;
//         }
//
//         let func = self.completer.clone();
//         orders.skip().perform_cmd(async move {
//             Msg::Fetched(func(string).await)
//         });
//     }
//
//     fn view_item(&self, index: usize, suggestion: &Suggestion) -> Node<Msg>
//     {
//         let is_active = self.selected == Some(index);
//         a![C!["dropdown-item", IF!(is_active => "is-active")],
//             mouse_ev(Ev::Click, move |_| Msg::SuggestionClick(index)),
//             &suggestion.name
//         ]
//     }
//
//     fn view_items(&self) -> Vec<Node<Msg>>
//     {
//         self.suggestions
//             .iter()
//             .enumerate()
//             .map(|(index, suggestion)| self.view_item(index, suggestion))
//             .collect()
//     }
//
//     fn view_dropdown(&self) -> Node<Msg>
//     {
//         div![C!["dropdown", IF!(self.is_open && !self.suggestions.is_empty() => "is-active")],
//             style!{St::Width => unit!(100, %)},
//             input![C!["input", IF!(self.is_ok => "is-success")],
//                 el_ref(&self.elem),
//                 attrs!{At::Type => "text"},
//                 input_ev(Ev::Input, Msg::InputChange),
//                 ev(Ev::Blur, |_| Msg::InputBlur),
//                 keyboard_ev(Ev::KeyDown, Msg::InputKeyDown),
//                 mouse_ev(Ev::Click, Msg::InputClick),
//             ],
//             span![C!["icon is-small", "is-right"],
//                 i![C!["fas fa-check"]]
//             ],
//             div![C!["dropdown-menu"],
//                 style!{St::Width => unit!(100, %)},
//                 ev(Ev::TouchStart, |_| Msg::SetIgnoreSuggestionBlur(true)),
//                 ev(Ev::MouseEnter, |_| Msg::SetIgnoreSuggestionBlur(true)),
//                 ev(Ev::MouseLeave, |_| Msg::SetIgnoreSuggestionBlur(false)),
//                 div![C!["dropdown-content"],
//                     Self::view_items(&self)
//                 ]
//             ]
//         ]
//     }
// }
