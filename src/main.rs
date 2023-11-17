use std::error::Error;

use nrs::app::Application;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    Application::new()?.run().await?;

    Ok(())
}
