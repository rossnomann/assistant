use refinery::{Error, Migration, Report, Runner};
use tokio_postgres::Client;

mod versions;

pub async fn run(client: &mut Client) -> Result<Report, Error> {
    let migrations: Result<Vec<Migration>, Error> = versions::build()
        .into_iter()
        .enumerate()
        .map(|(idx, version)| Migration::unapplied(&format!("U{}__{}", idx, version.name()), &version.build()))
        .collect();
    let runner = Runner::new(&migrations?);
    runner.run_async(client).await
}
