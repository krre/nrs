use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use sqlx;
use sqlx::postgres::PgPoolOptions;

use clap::Parser;
use log::info;

use crate::core::router;

#[derive(Parser, Debug)]
pub struct Config {
    #[clap(long, env)]
    port: u16,
    #[clap(long, env)]
    database_url: String,
    #[clap(long, env)]
    rust_log: String,
    #[clap(long, env)]
    jwt_secret: String,
}

pub struct Application {
    config: Config,
}

impl Application {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let config = Config::parse();
        Ok(Self { config })
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.config.database_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        info!(
            "Norm repository server started on port {}",
            self.config.port
        );

        let router = router::Router::new(pool, &self.config.jwt_secret);

        axum::Server::bind(&SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            self.config.port,
        ))
        .serve(router.into_make_service())
        .await?;
        Ok(())
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}
