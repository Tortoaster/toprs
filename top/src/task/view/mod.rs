use std::marker::PhantomData;
use std::ops::Deref;

use async_trait::async_trait;
use uuid::Uuid;

use top_derive::html;

use crate::html::event::{Change, Event, Feedback};
use crate::html::icon::Icon;
use crate::html::{Handler, Html, Refresh, ToHtml};
use crate::prelude::TaskValue;
use crate::share::{Share, ShareId, ShareRead, Shared};
use crate::task::Value;

pub mod convert;
pub mod generic;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct View<S: Share>(InnerView<S, S::Value>);

impl<T> View<Shared<T>>
where
    T: Clone + Send,
{
    pub fn new(value: T) -> Self {
        View::new_shared(Shared::new(TaskValue::Stable(value)))
    }
}

impl<S> View<S>
where
    S: Share,
{
    pub fn new_shared(share: S) -> Self {
        View(InnerView {
            id: Uuid::new_v4(),
            share,
            color: Color::default(),
            _type: PhantomData,
        })
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.0.color = color;
        self
    }
}

#[async_trait]
impl<S> Value for View<S>
where
    S: Share + Clone + Send + Sync,
    S::Value: Send + Sync,
{
    type Output = S::Value;
    type Share = S;

    async fn share(&self) -> Self::Share {
        self.0.share().await
    }

    async fn value(self) -> TaskValue<Self::Output> {
        self.0.value().await
    }
}

#[async_trait]
impl<S> Handler for View<S>
where
    S: Share + Send,
    S::Value: Send,
{
    async fn on_event(&mut self, event: Event) -> Feedback {
        self.0.on_event(event).await
    }
}

#[async_trait]
impl<S> Refresh for View<S>
where
    S: Share + ShareId + Send + Sync,
    S::Value: Send + Sync,
    InnerView<S, S::Value>: ToHtml,
{
    async fn refresh(&self, id: Uuid) -> Feedback {
        self.0.refresh(id).await
    }
}

#[async_trait]
impl<S> ToHtml for View<S>
where
    S: Share + Send + Sync,
    S::Value: Send + Sync,
    InnerView<S, S::Value>: ToHtml,
{
    async fn to_html(&self) -> Html {
        self.0.to_html().await
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct InnerView<S, T> {
    id: Uuid,
    share: S,
    color: Color,
    // Necessary for the `ToHtml` impls.
    _type: PhantomData<T>,
}

#[async_trait]
impl<S, T> Value for InnerView<S, T>
where
    S: Share<Value = T> + Clone + Send + Sync,
    T: Send + Sync,
{
    type Output = T;
    type Share = S;

    async fn share(&self) -> Self::Share {
        self.share.clone()
    }

    async fn value(self) -> TaskValue<Self::Output> {
        self.share.clone_value().await
    }
}

#[async_trait]
impl<S, T> Handler for InnerView<S, T>
where
    S: Send,
    T: Send,
{
    async fn on_event(&mut self, _event: Event) -> Feedback {
        Feedback::new()
    }
}

#[async_trait]
impl<S, T> Refresh for InnerView<S, T>
where
    Self: ToHtml,
    S: ShareId + Send + Sync,
    T: Send + Sync,
{
    async fn refresh(&self, id: Uuid) -> Feedback {
        if self.share.id() == id {
            Feedback::from(Change::Replace {
                id: self.id,
                html: self.to_html().await,
            })
        } else {
            Feedback::new()
        }
    }
}

macro_rules! impl_to_html {
    ($($ty:ty),*) => {
        $(
            #[async_trait]
            impl<S> ToHtml for InnerView<S, $ty>
            where
                S: ShareRead<Value = $ty> + Send + Sync,
            {
                async fn to_html(&self) -> Html {
                    let value = self.share.read().await;
                    html! {r#"
                        <div id="{self.id}">
                            <span style="color: {self.color};">{value.deref()}</span>
                        </div>
                    "#}
                }
            }
        )*
    };
}

impl_to_html!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    f32,
    f64,
    char,
    &'static str,
    String
);

#[async_trait]
impl<S> ToHtml for InnerView<S, bool>
where
    S: ShareRead<Value = bool> + Send + Sync,
{
    async fn to_html(&self) -> Html {
        match self.share.read().await.deref() {
            TaskValue::Stable(b) | TaskValue::Unstable(b) => {
                if *b {
                    Icon::Check.to_html().await
                } else {
                    Icon::XMark.to_html().await
                }
            }
            TaskValue::Empty => Html::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Color {
    Black,
    White,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Brown,
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

#[async_trait]
impl ToHtml for Color {
    async fn to_html(&self) -> Html {
        match self {
            Color::Black => Html("black".to_owned()),
            Color::White => Html("white".to_owned()),
            Color::Red => Html("red".to_owned()),
            Color::Orange => Html("orange".to_owned()),
            Color::Yellow => Html("yellow".to_owned()),
            Color::Green => Html("green".to_owned()),
            Color::Blue => Html("blue".to_owned()),
            Color::Purple => Html("purple".to_owned()),
            Color::Pink => Html("pink".to_owned()),
            Color::Brown => Html("brown".to_owned()),
        }
    }
}