use carapax::types::{Float, MessageData};
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::{error::Error, fmt, sync::Arc};
use tokio_postgres::{Client as PgClient, Error as PgError};

#[derive(Clone)]
pub struct NotesService {
    client: Arc<PgClient>,
}

impl NotesService {
    pub fn new(client: Arc<PgClient>) -> Self {
        Self { client }
    }

    pub async fn create(&self, note: &Note, keywords: Vec<String>) -> Result<(), NotesServiceError> {
        let data = serde_json::to_value(note).map_err(NotesServiceError::Serialize)?;
        self.client
            .execute(
                "INSERT INTO notes (data, keywords) VALUES ($1, $2)",
                &[&data, &keywords],
            )
            .await
            .map_err(NotesServiceError::Create)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum NotesServiceError {
    Create(PgError),
    Serialize(JsonError),
}

impl fmt::Display for NotesServiceError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NotesServiceError::*;
        match self {
            Create(err) => write!(out, "create note: {}", err),
            Serialize(err) => write!(out, "can not serialize note: {}", err),
        }
    }
}

impl Error for NotesServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::NotesServiceError::*;
        Some(match self {
            Create(err) => err,
            Serialize(err) => err,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Note {
    Animation { file_id: String },
    Audio { file_id: String },
    Document { file_id: String },
    Location { longitude: Float, latitude: Float },
    Photo { file_id: String },
    Text(String),
    Video { file_id: String },
    Voice { file_id: String },
}

impl TryFrom<MessageData> for Note {
    type Error = NoteError;

    fn try_from(data: MessageData) -> Result<Self, Self::Error> {
        Ok(match data {
            MessageData::Animation(animation) => Self::Animation {
                file_id: animation.file_id,
            },
            MessageData::Audio { data, .. } => Self::Audio { file_id: data.file_id },
            MessageData::Document { data, .. } => Self::Document { file_id: data.file_id },
            MessageData::Location(location) => Self::Location {
                latitude: location.latitude,
                longitude: location.longitude,
            },
            MessageData::Photo { data, .. } => Self::Photo {
                file_id: data
                    .into_iter()
                    .max_by(|x, y| (x.width, x.height).cmp(&(y.width, y.height)))
                    .map(|x| x.file_id)
                    .ok_or(NoteError::PhotoNotFound)?,
            },
            MessageData::Text(text) => Self::Text(text.data),
            MessageData::Video { data, .. } => Self::Video { file_id: data.file_id },
            MessageData::Voice { data, .. } => Self::Voice { file_id: data.file_id },
            _ => return Err(NoteError::UnsupportedMessage),
        })
    }
}

#[derive(Debug)]
pub enum NoteError {
    PhotoNotFound,
    UnsupportedMessage,
}

impl fmt::Display for NoteError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NoteError::*;
        match self {
            PhotoNotFound => write!(out, "could not find photo"),
            UnsupportedMessage => write!(out, "unsupported message"),
        }
    }
}

impl Error for NoteError {}
