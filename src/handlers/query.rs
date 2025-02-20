use std::{error::Error, fmt};

use carapax::{
    Ref,
    api::{Client, ExecuteError},
    types::{AnswerInlineQuery, InlineQuery, InlineQueryResult},
};

use crate::{
    entities::Keywords,
    services::{NotesService, NotesServiceError},
};

pub async fn handle(
    client: Ref<Client>,
    notes_service: Ref<NotesService>,
    input: InlineQuery,
) -> Result<(), QueryError> {
    let keywords = Keywords::from(input.query.split(' '));
    let notes = notes_service.query(keywords).await.map_err(QueryError::QueryNotes)?;
    let results: Vec<InlineQueryResult> = notes.into_iter().map(Into::into).collect();
    client.execute(AnswerInlineQuery::new(input.id, results)).await?;
    Ok(())
}

#[derive(Debug)]
pub enum QueryError {
    Execute(ExecuteError),
    QueryNotes(NotesServiceError),
}

impl From<ExecuteError> for QueryError {
    fn from(err: ExecuteError) -> Self {
        Self::Execute(err)
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::QueryError::*;
        match self {
            Execute(err) => err.fmt(out),
            QueryNotes(err) => err.fmt(out),
        }
    }
}

impl Error for QueryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::QueryError::*;
        Some(match self {
            Execute(err) => err,
            QueryNotes(err) => err,
        })
    }
}
