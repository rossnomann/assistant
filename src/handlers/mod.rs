use carapax::{dialogue::DialogueExt, Chain, CommandExt, CommandPredicate};

use crate::session::SessionBackend;

mod add;
mod list;
mod query;
mod remove;

pub fn setup() -> Chain {
    Chain::once()
        .with(query::handle)
        .with(list::handle.with_command("/list"))
        .with(remove::handle.with_command("/remove"))
        .with(add::handle.with_dialogue::<SessionBackend>(CommandPredicate::new("/add")))
}
