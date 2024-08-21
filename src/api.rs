use axum::{
    Json,
    Router,
    routing::{get},
};

use std::time::{Duration};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use crate::AppState;

pub fn build_router(start_state: AppState) -> Router {
    Router::new()
        .route("/api/v1/alarm", get(get_alarm).post(set_alarm))
        .route("/api/v1/activated", get(get_activated).post(set_activated))
        .with_state(start_state)
}


#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Alarm {
    #[serde(with = "humantime_serde")]
    pub end: Duration,
    #[serde(with = "humantime_serde")]
    pub fade_duration: Duration,
}


async fn set_alarm(
    State(state): State<AppState>,
    Query(params): Query<Alarm>,
) -> Result<(), StatusCode> {
    state.alarm.lock()
        .map(|mut alarm| { *alarm = params })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_alarm(State(state): State<AppState>) -> Result<Json<Alarm>, StatusCode> {
    state.alarm.lock()
        .map(|alarm| Json((*alarm).clone()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ActiveStatus {
    activated: bool,
}

async fn set_activated(
    State(state): State<AppState>,
    Query(params): Query<ActiveStatus>,
) -> Result<(), StatusCode> {
    state.activated.lock()
        .map(|mut activated| { *activated = params.activated })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_activated(State(state): State<AppState>) -> Result<Json<ActiveStatus>, StatusCode> {
    state.activated.lock()
        .map(|activated| Json(ActiveStatus { activated: *activated }))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}