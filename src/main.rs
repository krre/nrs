use std::error::Error;

use nrs::core::Application;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().format_timestamp(None).init();
    dotenv::dotenv().ok();

    Application::new()?.run().await?;

    Ok(())
}
