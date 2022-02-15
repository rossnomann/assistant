use carapax::{types::Update, Chain};

pub fn setup() -> Chain {
    Chain::once().add(handler)
}

async fn handler(update: Update) {
    println!("{:?}", update.get_user())
}
