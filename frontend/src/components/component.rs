use seed::prelude::Node;

pub trait Component
{
    type Msg: 'static;
    fn view(&self) -> Node<Self::Msg>;
}