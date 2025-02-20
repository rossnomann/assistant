use std::{error::Error, fmt};

use carapax::{
    Ref,
    api::{Client, ExecuteError},
    types::{ChatPeerId, Command, SendMessage},
};

use crate::services::{NotesService, NotesServiceError};

pub async fn handle(
    client: Ref<Client>,
    notes_service: Ref<NotesService>,
    command: Command,
    chat_id: ChatPeerId,
) -> Result<(), RemoveError> {
    let text = match command.get_args().first().map(|value| value.parse()).transpose() {
        Ok(Some(note_id)) => {
            if notes_service.remove(note_id).await.map_err(RemoveError::RemoveNote)? {
                "OK"
            } else {
                "Not found"
            }
        }
        Ok(None) => "Note ID is required",
        Err(_) => "Note ID is not an integer",
    };
    client.execute(SendMessage::new(chat_id, text)).await?;
    Ok(())
}

#[derive(Debug)]
pub enum RemoveError {
    Execute(ExecuteError),
    RemoveNote(NotesServiceError),
}

impl From<ExecuteError> for RemoveError {
    fn from(err: ExecuteError) -> Self {
        Self::Execute(err)
    }
}

impl fmt::Display for RemoveError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::RemoveError::*;
        match self {
            Execute(err) => err.fmt(out),
            RemoveNote(err) => err.fmt(out),
        }
    }
}

impl Error for RemoveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::RemoveError::*;
        Some(match self {
            Execute(err) => err,
            RemoveNote(err) => err,
        })
    }
}
