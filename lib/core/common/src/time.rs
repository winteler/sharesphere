use leptos::prelude::*;
use leptos_fluent::move_tr;

use crate::constants::{
    SECONDS_IN_DAY, SECONDS_IN_HOUR, SECONDS_IN_MINUTE, SECONDS_IN_MONTH, SECONDS_IN_YEAR,
};

enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
    Months,
    Years,
}

impl TimeUnit {
    pub fn to_localized_str(&self, count: i64, use_fullname: bool) -> Signal<String> {
        match (use_fullname, self) {
            (false, TimeUnit::Seconds) => move_tr!("time-seconds-short", {"count" => count}),
            (false, TimeUnit::Minutes) => move_tr!("time-minutes-short", {"count" => count}),
            (false, TimeUnit::Hours) => move_tr!("time-hours-short", {"count" => count}),
            (false, TimeUnit::Days) => move_tr!("time-days-short", {"count" => count}),
            (false, TimeUnit::Months) => move_tr!("time-months-short", {"count" => count}),
            (false, TimeUnit::Years) => move_tr!("time-years-short", {"count" => count}),
            (true, TimeUnit::Seconds) => move_tr!("time-seconds", {"count" => count}),
            (true, TimeUnit::Minutes) => move_tr!("time-minutes", {"count" => count}),
            (true, TimeUnit::Hours) => move_tr!("time-hours", {"count" => count}),
            (true, TimeUnit::Days) => move_tr!("time-days", {"count" => count}),
            (true, TimeUnit::Months) => move_tr!("time-months", {"count" => count}),
            (true, TimeUnit::Years) => move_tr!("time-years", {"count" => count}),
        }
    }
}

pub fn get_elapsed_time_string(
    timestamp: chrono::DateTime<chrono::Utc>,
    use_fullname: bool,
) -> Signal<String> {
    let elapsed_time = chrono::Utc::now().signed_duration_since(timestamp);
    let seconds = elapsed_time.num_seconds();
    match seconds {
        seconds if seconds < SECONDS_IN_MINUTE => TimeUnit::Seconds.to_localized_str(seconds, use_fullname),
        seconds if seconds < SECONDS_IN_HOUR => TimeUnit::Minutes.to_localized_str(seconds / SECONDS_IN_MINUTE, use_fullname),
        seconds if seconds < SECONDS_IN_DAY => TimeUnit::Hours.to_localized_str(seconds / SECONDS_IN_HOUR, use_fullname),
        seconds if seconds < SECONDS_IN_MONTH => TimeUnit::Days.to_localized_str(seconds / SECONDS_IN_DAY, use_fullname),
        seconds if seconds < SECONDS_IN_YEAR => TimeUnit::Months.to_localized_str(seconds / SECONDS_IN_MONTH, use_fullname),
        _ => TimeUnit::Years.to_localized_str(seconds / SECONDS_IN_YEAR, use_fullname),
    }
}