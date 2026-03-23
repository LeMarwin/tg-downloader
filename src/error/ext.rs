//! Error extension and handler for teloxide
//! Annotate errors with `with_chat(chat_id)` to send an error message to the user.
//! All erros, with or without `chat_id` are traced in the log

use std::sync::Arc;

use teloxide::{Bot, error_handlers::ErrorHandler, prelude::Requester as _, types::ChatId};

/// Handler result alias
pub type HandlerResult<T> = Result<T, BoxedError>;

/// Catch-all error type
#[derive(Debug)]
pub struct BoxedError {
    chat_id: Option<ChatId>,
    error: Box<dyn std::error::Error + Send + Sync + 'static>,
}

/// Extension for Errors
pub trait ErrorExt<T, E>: Sized {
    /// Optionally add a [`ChatId`] (it might not be present in the message)
    fn with_chat_opt(self, chat_id: Option<ChatId>) -> Result<T, BoxedError>;

    /// Add [`ChatId`] to an error, so that it's sent to the user
    fn with_chat(self, chat_id: ChatId) -> Result<T, BoxedError> {
        self.with_chat_opt(Some(chat_id))
    }
}

impl<T, E: std::error::Error + Send + Sync + 'static> ErrorExt<T, E> for Result<T, E> {
    fn with_chat_opt(self, chat_id: Option<ChatId>) -> Result<T, BoxedError> {
        self.map_err(|e| BoxedError {
            chat_id,
            error: Box::new(e),
        })
    }
}

impl<E: std::error::Error + Send + Sync + 'static> From<E> for BoxedError {
    fn from(value: E) -> Self {
        Self {
            chat_id: None,
            error: Box::new(value),
        }
    }
}

/// Error handler for teloxide
///
/// Sends erorrs that have [`ChatId`] attached to the specified user
pub struct ErrorSender {
    bot: Arc<Bot>,
}

impl ErrorSender {
    /// Create an [`ErrorSender`] with a given bot
    pub fn with_bot(bot: Arc<Bot>) -> Arc<dyn ErrorHandler<BoxedError> + Send + Sync + 'static> {
        Arc::from(Self { bot })
    }
}

impl ErrorHandler<BoxedError> for ErrorSender {
    fn handle_error(
        self: Arc<Self>,
        BoxedError { chat_id, error }: BoxedError,
    ) -> futures::future::BoxFuture<'static, ()> {
        tracing::error!(?chat_id, ?error);
        Box::pin(async move {
            if let Some(chat_id) = chat_id {
                drop(self.bot.send_message(chat_id, error.to_string()).await);
            }
        })
    }
}
//
