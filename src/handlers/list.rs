use std::{error::Error, fmt};

use carapax::{
    Ref,
    api::{Client, ExecuteError},
    types::{Message, ParseMode, SendMessage},
};

use crate::services::{NotesService, NotesServiceError};

pub async fn handle(client: Ref<Client>, notes_service: Ref<NotesService>, message: Message) -> Result<(), ListError> {
    let chat_id = message.chat.get_id();
    let items: Vec<String> = notes_service.get_list().await.map_err(ListError::GetNotes)?.collect();
    if items.is_empty() {
        client.execute(SendMessage::new(chat_id, "There are no items")).await?;
    } else {
        for item in items {
            client
                .execute(SendMessage::new(chat_id, item).with_parse_mode(ParseMode::MarkdownV2))
                .await?;
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum ListError {
    Execute(ExecuteError),
    GetNotes(NotesServiceError),
}

impl From<ExecuteError> for ListError {
    fn from(err: ExecuteError) -> Self {
        Self::Execute(err)
    }
}

impl fmt::Display for ListError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ListError::*;
        match self {
            Execute(err) => err.fmt(out),
            GetNotes(err) => err.fmt(out),
        }
    }
}

impl Error for ListError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::ListError::*;
        Some(match self {
            Execute(err) => err,
            GetNotes(err) => err,
        })
    }
}
