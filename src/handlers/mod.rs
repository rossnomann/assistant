use crate::session::SessionBackend;
use carapax::{dialogue::DialogueExt, Chain, CommandPredicate};

mod add;

pub fn setup() -> Chain {
    Chain::once().add(add::handle.dialogue::<SessionBackend>(CommandPredicate::new("/add")))
}
