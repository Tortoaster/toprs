use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::html::Html;
use crate::id::Id;

/// Interaction event from the user, such as checking a checkbox or pressing a button.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Event {
    Update { id: Id, value: String },
    Press { id: Id },
}

#[async_trait]
pub trait EventHandler {
    async fn receive(&mut self) -> Option<Result<Event, EventError>>;
}

#[derive(Debug, Error)]
pub enum EventError {
    #[error("error during deserialization: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("failed to receive event")]
    Receive,
}

/// Changes to the user interface in response to [`Event`]s, such as confirming a value is valid, or
/// replacing the content after the user presses a button.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Feedback {
    /// Replace this element's content with new html.
    Insert { id: Id, html: Html },
    /// Replace this element with new html.
    Replace { id: Id, html: Html },
    /// Add html to this element.
    Append { id: Id, html: Html },
    /// Remove this element.
    Remove { id: Id },
    /// The value of this html is valid.
    Valid { id: Id },
    /// The value of this html is invalid.
    Invalid { id: Id },
}

#[async_trait]
pub trait FeedbackHandler {
    async fn send(&mut self, feedback: Feedback) -> Result<(), FeedbackError>;
}

#[derive(Debug, Error)]
pub enum FeedbackError {
    #[error("error during serialization: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("failed to send feedback")]
    Send,
}
