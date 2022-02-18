use crate::entities::{NewNote, NoteDataError, NoteInfoList};
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
                &[&data, &note.keywords()],
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
}

#[derive(Debug)]
pub enum NotesServiceError {
    Create(PgError),
    GetList(PgError),
    Remove(PgError),
    Serialize(NoteDataError),
}

impl fmt::Display for NotesServiceError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::NotesServiceError::*;
        match self {
            Create(err) => write!(out, "create note: {}", err),
            GetList(err) => write!(out, "get notes: {}", err),
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
            Remove(err) => err,
            Serialize(err) => err,
        })
    }
}
