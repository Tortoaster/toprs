//! This module contains functionality for generating user interfaces for tasks.

use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use futures::future;
use serde::Serialize;
use uuid::Uuid;

use top_derive::html;

use crate::task::TaskValue;

pub mod event;
pub mod icon;

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(transparent)]
pub struct Html(pub String);

impl Html {
    pub async fn wrapper(title: &str) -> Html {
        html! {r#"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1">
                    <title>{title}</title>
                    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.3/css/bulma.min.css">
                    <script src="https://kit.fontawesome.com/e94af86b8c.js" crossorigin="anonymous"></script>
                    <script src="top/top.js"></script>
                </head>
                <body>
                    <section class="section">
                        <div class="container">
                            <div id="00000000-0000-0000-0000-000000000000"></div>
                        </div>
                    </section>
                </body>
            </html>
        "#}
    }
}

impl Display for Html {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromIterator<Html> for Html {
    fn from_iter<T: IntoIterator<Item = Html>>(iter: T) -> Self {
        let html: String = iter.into_iter().map(|html| html.0).collect();
        Html(html)
    }
}

#[async_trait]
pub trait ToRepr<T> {
    async fn to_repr(&self) -> T;
}

macro_rules! impl_to_html {
    ($($ty:ty),*) => {
        $(
            #[async_trait]
            impl ToRepr<Html> for $ty {
                async fn to_repr(&self) -> Html {
                    Html(self.to_string())
                }
            }
        )*
    };
}

impl_to_html!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool, char, &str,
    String, Uuid
);

#[async_trait]
impl ToRepr<Html> for Html {
    async fn to_repr(&self) -> Html {
        self.clone()
    }
}

#[async_trait]
impl<T> ToRepr<Html> for Option<T>
where
    T: ToRepr<Html> + Sync,
{
    async fn to_repr(&self) -> Html {
        match self {
            None => Html::default(),
            Some(value) => value.to_repr().await,
        }
    }
}

#[async_trait]
impl<T, E> ToRepr<Html> for Result<T, E>
where
    T: ToRepr<Html> + Sync,
    E: Sync,
{
    async fn to_repr(&self) -> Html {
        match self {
            Ok(value) => value.to_repr().await,
            Err(_) => Html::default(),
        }
    }
}

#[async_trait]
impl<T> ToRepr<Html> for TaskValue<T>
where
    T: ToRepr<Html> + Sync,
{
    async fn to_repr(&self) -> Html {
        match self {
            TaskValue::Stable(value) | TaskValue::Unstable(value) => value.to_repr().await,
            TaskValue::Empty => Html::default(),
        }
    }
}

#[async_trait]
impl<T> ToRepr<Html> for Vec<T>
where
    T: ToRepr<Html> + Sync,
{
    async fn to_repr(&self) -> Html {
        future::join_all(self.iter().map(ToRepr::to_repr))
            .await
            .into_iter()
            .collect()
    }
}
