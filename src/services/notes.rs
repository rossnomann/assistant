use carapax::types::{Float, MessageData};
use serde::{Deserialize, Serialize};
use serde_json::Error as JsonError;
use std::{collections::HashMap, error::Error, fmt, sync::Arc};
use tokio_postgres::{Client as PgClient, Error as PgError, Row};

const MAX_LIST_ITEM_LEN: usize = 4096;

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

    pub async fn get_list(&self) -> Result<NoteInfoList, NotesServiceError> {
        self.client
            .query("SELECT id, keywords FROM notes ORDER BY id ASC", &[])
            .await
            .map(NoteInfoList::from)
            .map_err(NotesServiceError::GetList)
    }
}

pub struct NoteInfoList {
    items: Vec<NoteInfo>,
    current_index: usize,
}

impl NoteInfoList {
    fn new(items: Vec<NoteInfo>) -> Self {
        Self {
            items,
            current_index: 0,
        }
    }
}

impl Iterator for NoteInfoList {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let total_items = self.items.len();
        if self.current_index >= total_items {
            None
        } else {
            let mut size = 0;
            let mut result = Vec::new();
            for idx in self.current_index..total_items {
                let item = self.items[idx].as_string();
                let item_len = item.len();
                if size + item_len > MAX_LIST_ITEM_LEN {
                    break;
                }
                result.push(item);
                size += item_len;
                self.current_index += 1;
            }
            Some(result.join("\n"))
        }
    }
}

impl From<Vec<Row>> for NoteInfoList {
    fn from(rows: Vec<Row>) -> Self {
        let items = rows.into_iter().map(NoteInfo::from).collect();
        Self::new(items)
    }
}

#[derive(Debug)]
pub struct NoteInfo {
    id: i32,
    keywords: Vec<String>,
}

impl NoteInfo {
    fn as_string(&self) -> String {
        let mut result = format!(r#"`{}` \- {}"#, self.id, self.keywords.join(" "));
        if result.len() > MAX_LIST_ITEM_LEN {
            result = result.chars().take(MAX_LIST_ITEM_LEN - 3).collect();
            result.push_str("...");
        }
        result
    }
}

impl From<Row> for NoteInfo {
    fn from(row: Row) -> Self {
        let indexes: HashMap<&str, usize> = row
            .columns()
            .iter()
            .enumerate()
            .map(|(idx, column)| (column.name(), idx))
            .collect();
        Self {
            id: row.get(indexes["id"]),
            keywords: row.get(indexes["keywords"]),
        }
    }
}

#[derive(Debug)]
pub enum NotesServiceError {
    Create(PgError),
    GetList(PgError),
    Serialize(JsonError),
}

impl fmt::Display for NotesServiceError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NotesServiceError::*;
        match self {
            Create(err) => write!(out, "create note: {}", err),
            GetList(err) => write!(out, "get list: {}", err),
            Serialize(err) => write!(out, "can not serialize note: {}", err),
        }
    }
}

impl Error for NotesServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::NotesServiceError::*;
        Some(match self {
            Create(err) => err,
            GetList(err) => err,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_note_info<T, I>(id: i32, keywords: T) -> NoteInfo
    where
        T: IntoIterator<Item = I>,
        I: Into<String>,
    {
        NoteInfo {
            id,
            keywords: keywords.into_iter().map(Into::into).collect(),
        }
    }

    #[test]
    fn note_info_list() {
        let list = NoteInfoList::new(vec![
            create_note_info(1, vec!["k1", "k2"]),
            create_note_info(2, vec!["k3", "k4"]),
            create_note_info(3, vec!["k".repeat(MAX_LIST_ITEM_LEN * 2)]),
        ]);
        let formatted_list: Vec<String> = list.collect();
        const PREFIX_LEN: usize = 7;
        assert_eq!(
            formatted_list,
            &[
                String::from("`1` \\- k1 k2\n`2` \\- k3 k4"),
                format!(r#"`3` \- {}..."#, "k".repeat(MAX_LIST_ITEM_LEN - 3 - PREFIX_LEN))
            ]
        )
    }
}
