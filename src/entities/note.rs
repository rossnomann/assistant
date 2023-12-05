use std::{error::Error, fmt};

use carapax::types::{
    Float, InlineQueryResult, InlineQueryResultArticle, InlineQueryResultCachedDocument, InlineQueryResultCachedGif,
    InlineQueryResultCachedPhoto, InlineQueryResultCachedVideo, InlineQueryResultCachedVoice,
    InlineQueryResultLocation, MessageData,
};
use serde::{Deserialize, Serialize};
use serde_json::{Error as JsonError, Value as JsonValue};
use tokio_postgres::Row;

use crate::entities::Keywords;

#[derive(Debug)]
pub struct NewNote {
    data: NoteData,
    keywords: Keywords,
}

impl NewNote {
    pub fn data(&self) -> &NoteData {
        &self.data
    }

    pub fn keywords(&self) -> &Keywords {
        &self.keywords
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NoteData {
    Animation { file_id: String },
    Audio { file_id: String },
    Document { file_id: String },
    Location { longitude: Float, latitude: Float },
    Photo { file_id: String },
    Text(String),
    Video { file_id: String },
    Voice { file_id: String },
}

impl NoteData {
    pub fn into_new(self, keywords: Keywords) -> NewNote {
        NewNote { data: self, keywords }
    }

    pub fn as_json(&self) -> Result<JsonValue, NoteDataError> {
        serde_json::to_value(self).map_err(NoteDataError::Serialize)
    }
}

impl TryFrom<MessageData> for NoteData {
    type Error = NoteDataError;

    fn try_from(data: MessageData) -> Result<Self, Self::Error> {
        Ok(match data {
            MessageData::Animation(animation) => Self::Animation {
                file_id: animation.file_id,
            },
            MessageData::Audio(audio) => Self::Audio {
                file_id: audio.data.file_id,
            },
            MessageData::Document(document) => Self::Document {
                file_id: document.data.file_id,
            },
            MessageData::Location(location) => Self::Location {
                latitude: location.latitude,
                longitude: location.longitude,
            },
            MessageData::Photo(photo) => Self::Photo {
                file_id: photo
                    .data
                    .into_iter()
                    .max_by(|x, y| (x.width, x.height).cmp(&(y.width, y.height)))
                    .map(|x| x.file_id)
                    .ok_or(NoteDataError::PhotoNotFound)?,
            },
            MessageData::Text(text) => Self::Text(text.data),
            MessageData::Video(video) => Self::Video {
                file_id: video.data.file_id,
            },
            MessageData::Voice(voice) => Self::Voice {
                file_id: voice.data.file_id,
            },
            _ => return Err(NoteDataError::UnsupportedMessage),
        })
    }
}

#[derive(Debug)]
pub enum NoteDataError {
    PhotoNotFound,
    Serialize(JsonError),
    UnsupportedMessage,
}

impl fmt::Display for NoteDataError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NoteDataError::*;
        match self {
            PhotoNotFound => write!(out, "could not find photo"),
            Serialize(err) => write!(out, "serialize note data: {err}"),
            UnsupportedMessage => write!(out, "can not create note data from provided message"),
        }
    }
}

impl Error for NoteDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::NoteDataError::*;
        match self {
            PhotoNotFound => None,
            Serialize(err) => Some(err),
            UnsupportedMessage => None,
        }
    }
}

#[derive(Debug)]
pub struct Note {
    id: i32,
    data: NoteData,
    keywords: Keywords,
}

impl TryFrom<Row> for Note {
    type Error = NoteError;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let data: JsonValue = row.get("data");
        let keywords: Vec<String> = row.get("keywords");
        Ok(Self {
            id: row.get("id"),
            data: serde_json::from_value(data).map_err(NoteError::Deserialize)?,
            keywords: Keywords::from(keywords),
        })
    }
}

impl From<Note> for InlineQueryResult {
    fn from(note: Note) -> Self {
        let id = format!("{}", note.id);
        let title = note.keywords.as_string();
        match note.data {
            NoteData::Animation { file_id } => InlineQueryResultCachedGif::new(file_id, id).with_title(title).into(),
            NoteData::Audio { file_id } => InlineQueryResultCachedDocument::new(file_id, id, title).into(),
            NoteData::Document { file_id } => InlineQueryResultCachedDocument::new(file_id, id, title).into(),
            NoteData::Location { latitude, longitude } => {
                InlineQueryResultLocation::new(id, latitude, longitude, title).into()
            }
            NoteData::Photo { file_id } => InlineQueryResultCachedPhoto::new(id, file_id).with_title(title).into(),
            NoteData::Text(text) => InlineQueryResultArticle::new(id, text, title).into(),
            NoteData::Video { file_id } => InlineQueryResultCachedVideo::new(id, title, file_id).into(),
            NoteData::Voice { file_id } => InlineQueryResultCachedVoice::new(id, title, file_id).into(),
        }
    }
}

#[derive(Debug)]
pub enum NoteError {
    Deserialize(JsonError),
}

impl fmt::Display for NoteError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NoteError::*;
        match self {
            Deserialize(err) => write!(out, "deserialize note: {err}"),
        }
    }
}

impl Error for NoteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::NoteError::*;
        match self {
            Deserialize(err) => Some(err),
        }
    }
}
