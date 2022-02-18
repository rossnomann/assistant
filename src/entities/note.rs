use carapax::types::{Float, MessageData};
use serde::{Deserialize, Serialize};
use serde_json::{Error as JsonError, Value as JsonValue};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct NewNote {
    data: NoteData,
    keywords: Vec<String>,
}

impl NewNote {
    pub fn data(&self) -> &NoteData {
        &self.data
    }

    pub fn keywords(&self) -> &[String] {
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
    pub fn into_new<K, KI>(self, keywords: K) -> NewNote
    where
        K: IntoIterator<Item = KI>,
        KI: Into<String>,
    {
        NewNote {
            data: self,
            keywords: keywords.into_iter().map(Into::into).collect(),
        }
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
                    .ok_or(NoteDataError::PhotoNotFound)?,
            },
            MessageData::Text(text) => Self::Text(text.data),
            MessageData::Video { data, .. } => Self::Video { file_id: data.file_id },
            MessageData::Voice { data, .. } => Self::Voice { file_id: data.file_id },
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
            Serialize(err) => write!(out, "serialize note data: {}", err),
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
