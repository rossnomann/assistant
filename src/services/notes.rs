use crate::entities::{Keywords, NewNote, Note, NoteDataError, NoteError, NoteInfoList};
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

    pub async fn create(&self, note: NewNote) -> Result<(), NotesServiceError> {
        let data = note.data().as_json().map_err(NotesServiceError::Serialize)?;
        self.client
            .execute(
                "INSERT INTO notes (data, keywords) VALUES ($1, $2)",
                &[&data, &note.keywords().as_ref()],
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

    pub async fn remove(&self, id: i32) -> Result<bool, NotesServiceError> {
        self.client
            .execute("DELETE FROM notes WHERE id = $1", &[&id])
            .await
            .map(|affected_rows| affected_rows != 0)
            .map_err(NotesServiceError::Remove)
    }

    pub async fn query(&self, keywords: Keywords) -> Result<Vec<Note>, NotesServiceError> {
        let rows = self
            .client
            .query("SELECT * FROM notes WHERE keywords @> $1", &[&keywords.as_ref()])
            .await
            .map_err(NotesServiceError::Query)?;
        rows.into_iter()
            .map(Note::try_from)
            .collect::<Result<Vec<Note>, NoteError>>()
            .map_err(NotesServiceError::MapNote)
    }
}

#[derive(Debug)]
pub enum NotesServiceError {
    Create(PgError),
    GetList(PgError),
    MapNote(NoteError),
    Query(PgError),
    Remove(PgError),
    Serialize(NoteDataError),
}

impl fmt::Display for NotesServiceError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NotesServiceError::*;
        match self {
            Create(err) => write!(out, "create note: {}", err),
            GetList(err) => write!(out, "get notes: {}", err),
            MapNote(err) => write!(out, "map note: {}", err),
            Query(err) => write!(out, "query notes: {}", err),
            Remove(err) => write!(out, "remove note: {}", err),
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
            MapNote(err) => err,
            Query(err) => err,
            Remove(err) => err,
            Serialize(err) => err,
        })
    }
}
