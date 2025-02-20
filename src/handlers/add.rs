use std::{error::Error, fmt};

use carapax::{
    Ref,
    api::{Client, ExecuteError},
    dialogue::{DialogueInput, DialogueResult, DialogueState},
    types::{ChatPeerId, Message, SendMessage},
};
use serde::{Deserialize, Serialize};

use crate::{
    entities::{Keywords, NoteData},
    services::{NotesService, NotesServiceError},
    session::SessionBackend,
};

pub async fn handle(
    client: Ref<Client>,
    notes_service: Ref<NotesService>,
    chat_id: ChatPeerId,
    input: DialogueInput<AddState, SessionBackend>,
    message: Message,
) -> Result<DialogueResult<AddState>, AddError> {
    Ok(match input.state {
        AddState::Start => {
            client.execute(SendMessage::new(chat_id, "Send any message")).await?;
            AddState::SetMessage
        }
        AddState::SetMessage => {
            let note = match NoteData::try_from(message.data) {
                Ok(note) => note,
                Err(err) => {
                    client.execute(SendMessage::new(chat_id, err.to_string())).await?;
                    return Ok(AddState::SetMessage.into());
                }
            };
            client.execute(SendMessage::new(chat_id, "Send keywords")).await?;
            AddState::SetKeywords(note)
        }
        AddState::SetKeywords(note_data) => {
            let keywords = match message.get_text() {
                Some(text) => Keywords::from(text.data.split(' ')),
                None => {
                    client.execute(SendMessage::new(chat_id, "Done")).await?;
                    return Ok(AddState::SetKeywords(note_data).into());
                }
            };
            notes_service
                .create(note_data.into_new(keywords))
                .await
                .map_err(AddError::CreateNote)?;
            client.execute(SendMessage::new(chat_id, "Done")).await?;
            return Ok(DialogueResult::Exit);
        }
    }
    .into())
}

#[derive(Default, Serialize, Deserialize)]
pub enum AddState {
    #[default]
    Start,
    SetMessage,
    SetKeywords(NoteData),
}

impl DialogueState for AddState {
    fn dialogue_name() -> &'static str {
        "add"
    }
}

#[derive(Debug)]
pub enum AddError {
    Execute(ExecuteError),
    CreateNote(NotesServiceError),
}

impl From<ExecuteError> for AddError {
    fn from(err: ExecuteError) -> Self {
        Self::Execute(err)
    }
}

impl fmt::Display for AddError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::AddError::*;
        match self {
            Execute(err) => err.fmt(out),
            CreateNote(err) => err.fmt(out),
        }
    }
}

impl Error for AddError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::AddError::*;
        Some(match self {
            Execute(err) => err,
            CreateNote(err) => err,
        })
    }
}
