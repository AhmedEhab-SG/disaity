use chrono::{Local, Timelike};
use reqwest::Client;
use serde::Deserialize;
use serenity::all::{Context as SerenityContext, CreateMessage};
use serenity::builder::CreateAllowedMentions;
use std::collections::HashMap;
use std::time::Duration;

use crate::commands::Subscription;

#[derive(Deserialize)]
struct AladhanResponse {
    data: PrayerData,
}

#[derive(Deserialize)]
struct PrayerData {
    timings: HashMap<String, String>,
}

pub fn start_prayer_loop(ctx: SerenityContext, subscription: Subscription, http: Client) {
    tokio::spawn(async move {
        let mut last_notified_minute = 99;

        loop {
            let now = Local::now();
            let current_minute = now.minute();

            if current_minute != last_notified_minute {
                let current_time_str = now.format("%H:%M").to_string();
                let subs_guard = subscription.prayer_subscription.read().await;

                for sub in subs_guard.subscription.values() {
                    let url = format!(
                        "https://api.aladhan.com/v1/timingsByCity?city={}&country={}&method=5",
                        sub.city, sub.country
                    );

                    if let Ok(res) = http.get(&url).send().await {
                        if let Ok(json) = res.json::<AladhanResponse>().await {
                            let timings = json.data.timings;
                            let prayers_to_check = ["Fajr", "Dhuhr", "Asr", "Maghrib", "Isha"];

                            for prayer in prayers_to_check {
                                if let Some(prayer_time) = timings.get(prayer) {
                                    let clean_prayer_time =
                                        prayer_time.split_whitespace().next().unwrap_or("");

                                    if clean_prayer_time == current_time_str {
                                        last_notified_minute = current_minute;

                                        let mention_prefix = match sub.role_id {
                                            Some(role_id) => format!("<@&{}> ", role_id),
                                            None => String::new(),
                                        };

                                        let text_content = format!(
                                            "{}🕌 **It's time for {}** in {}, {}!**",
                                            mention_prefix, prayer, sub.city, sub.country
                                        );

                                        let mut message_payload =
                                            CreateMessage::new().content(text_content);

                                        if let Some(role_id) = sub.role_id {
                                            let allowed_mentions =
                                                CreateAllowedMentions::new().roles(vec![role_id]);
                                            message_payload =
                                                message_payload.allowed_mentions(allowed_mentions);
                                        }

                                        sub.channel_id
                                            .send_message(&ctx.http, message_payload)
                                            .await
                                            .ok();
                                    }
                                }
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}
