use assistant::run;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();
    if let Err(err) = run().await {
        log::error!("{}", err)
    }
}
