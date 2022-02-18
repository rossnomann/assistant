use crate::session::SessionBackend;
use carapax::{dialogue::DialogueExt, Chain, CommandExt, CommandPredicate};

mod add;
mod list;
mod remove;

pub fn setup() -> Chain {
    Chain::once()
        .add(list::handle.command("/list"))
        .add(remove::handle.command("/remove"))
        .add(add::handle.dialogue::<SessionBackend>(CommandPredicate::new("/add")))
}
