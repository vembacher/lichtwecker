mod api;
mod light_control;

use std::sync::{Arc, Mutex};


use tracing::{info};
use clap::Parser;
use url::Url;
use crate::api::{Alarm, build_router};
use crate::light_control::run_alarm;


/// Lichtwecker
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    end: humantime::Duration,
    #[arg(long)]
    fade_duration: humantime::Duration,
    #[arg(long)]
    url: Url,
    #[arg(long)]
    api_key: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub alarm: Arc<Mutex<Alarm>>,
    pub activated: Arc<Mutex<bool>>,
    pub url: Arc<Mutex<Url>>,
    pub api_key: Arc<Mutex<String>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_subscriber::fmt().init();

    let state = AppState {
        alarm: Arc::new(Mutex::new(Alarm {
            end: args.end.into(),
            fade_duration: args.fade_duration.into(),
        })),
        activated: Arc::new(Mutex::new(true)),
        url: Arc::new(Mutex::new(args.url)),
        api_key: Arc::new(Mutex::new(args.api_key)),
    };


    let state_api = state.clone();
    let app = build_router(state_api);
    tokio::spawn(async move {
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .expect("failed to start app");
    });

    loop {
        let state_alarm = state.clone();
        let alarm_handle = tokio::spawn(async move {
            run_alarm(state_alarm).await
        });

        match alarm_handle.await {
            Ok(res) => { info!("{res:?}") }
            Err(err) => { info!("failed to join alarm task {err}") }
        }
    }
}


