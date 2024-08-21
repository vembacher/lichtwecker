use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Local, NaiveTime, Timelike};
use tokio::time::{Instant, sleep, sleep_until};
use deconz_rs::connection::{DeconzConnection, RequestResponse};
use tracing::{debug, info};
use crate::{AppState};

const N: isize = 256;

pub(crate) async fn run_alarm(
    state: AppState,
) -> Result<()> {
    loop {
        let alarm = state.alarm.lock()
            .map(|alarm| (*alarm).clone())
            .map_err(|_| anyhow!("failed to get lock for alarm"))?;
        let connection = state.url.try_lock()
            .map_err(|_| anyhow!("failed to get lock on url"))
            .map(|url| url.clone())
            .and_then(|url| state.api_key.try_lock()
                .map_err(|_| anyhow!("failed to get lock on url"))
                .map(|api_key| (url, api_key.clone()))
            )
            .and_then(|(url, api_key)| DeconzConnection::new(url, api_key)
                .map_err(|err| anyhow!("failed to create DeConz connection: {err}")))?;

        let mut lights = connection.get_all_lights().await.unwrap();
        let now: DateTime<Local> = Local::now();
        let wakeup_time = NaiveTime::from_num_seconds_from_midnight_opt((alarm.end).as_secs() as u32, 0).unwrap();
        let fade_duration = Duration::from_std(alarm.fade_duration)
            .context("failed to convert duration from std to chrono duration")?;
        let start = wakeup_time - fade_duration;


        let t_zero = if wakeup_time < now.time() {
            now + Duration::hours(24)
        } else { now };

        let end = t_zero
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .map(|t| t + Duration::from_std(alarm.end).unwrap()).expect("failed time conversion");
        let start = end - Duration::from_std(alarm.fade_duration).unwrap();

        let delta = if now < start { start - now } else { Duration::hours(0) };
        eprintln!("start: {start:?}");
        eprintln!("start: {end:?}");

        eprintln!("delta: {delta:?}");

        eprintln!("sleeping");
        sleep(delta.to_std().unwrap()).await;

        let chunk_delta = Duration::microseconds(fade_duration.num_microseconds().unwrap() / N as i64);

        for (id, mut light) in lights.iter_mut() {
            light
                .change_brightness(-254)
                .on(true);
            light.state.ct = light.ct_max;
            debug!("State of ceiling light after updating: {:?}", light.state);
            let response = connection.set_light_state(id.as_str(), &light.state)
                .await.unwrap();
            debug!("Response for ceiling: {:?}",response);
        }


        for i in 0..N {
            if !*state.activated.lock().map_err(|_| anyhow!("failed to lock mutex"))? {
                info!("exiting loop, deactivated");
                return Ok(());
            }
            sleep(chunk_delta.to_std().unwrap()).await;
            info!("{i} Sleeping complete");
            info!("{i} Setting lights...");
            for (id, light) in lights.iter_mut() {
                light
                    .change_brightness(1)
                    .change_color_temperature(-1);
                match connection.set_light_state(id, &light.state).await {
                    Ok(response) => {
                        response.iter().for_each(|att| {
                            match att {
                                RequestResponse::Error { address, description, r#type } => {
                                    debug!("{address} failed with '{description}' (code: {:?})", r#type)
                                }
                                RequestResponse::Success(success) => {
                                    debug!("{success:?}");
                                }
                            }
                        })
                    }
                    Err(err) => {
                        debug!("{err}");
                    }
                }
            }
        }
    }
}