use crate::{
    entities::NoteData,
    services::{NotesService, NotesServiceError},
    session::SessionBackend,
};
use carapax::{
    dialogue::{DialogueInput, DialogueResult, DialogueState},
    methods::SendMessage,
    types::{ChatId, Message},
    Api, ExecuteError, Ref,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub async fn handle(
    api: Ref<Api>,
    notes_service: Ref<NotesService>,
    chat_id: ChatId,
    input: DialogueInput<AddState, SessionBackend>,
    message: Message,
) -> Result<DialogueResult<AddState>, AddError> {
    Ok(match input.state {
        AddState::Start => {
            api.execute(SendMessage::new(chat_id, "Send any message")).await?;
            AddState::SetMessage
        }
        AddState::SetMessage => {
            let note = match NoteData::try_from(message.data) {
                Ok(note) => note,
                Err(err) => {
                    api.execute(SendMessage::new(chat_id, err.to_string())).await?;
                    return Ok(AddState::SetMessage.into());
                }
            };
            api.execute(SendMessage::new(chat_id, "Send keywords")).await?;
            AddState::SetKeywords(note)
        }
        AddState::SetKeywords(note_data) => {
            let keywords: Vec<String> = match message.get_text() {
                Some(text) => text.data.split(' ').map(String::from).collect(),
                None => {
                    api.execute(SendMessage::new(chat_id, "Done")).await?;
                    return Ok(AddState::SetKeywords(note_data).into());
                }
            };
            notes_service
                .create(note_data.into_new(keywords))
                .await
                .map_err(AddError::CreateNote)?;
            api.execute(SendMessage::new(chat_id, "Done")).await?;
            return Ok(DialogueResult::Exit);
        }
    }
    .into())
}

#[derive(Serialize, Deserialize)]
pub enum AddState {
    Start,
    SetMessage,
    SetKeywords(NoteData),
}

impl Default for AddState {
    fn default() -> Self {
        AddState::Start
    }
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
