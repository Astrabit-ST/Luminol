// Copyright (C) 2022 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime},
};

pub struct DiscordClient {
    discord: Arc<discord_sdk::Discord>,
    activity_promise: Option<poll_promise::Promise<Instant>>,
    start_time: SystemTime,
}

const APP_ID: discord_sdk::AppId = 1024186911478263830;

impl Default for DiscordClient {
    fn default() -> Self {
        let (_wheel, handler) =
            discord_sdk::wheel::Wheel::new(Box::new(|_| panic!("discord error")));

        let discord = discord_sdk::Discord::new(
            discord_sdk::DiscordApp::PlainId(APP_ID),
            discord_sdk::Subscriptions::all(),
            Box::new(handler),
        )
        .expect("Failed to create discord");

        Self {
            discord: Arc::new(discord),
            activity_promise: None,
            start_time: SystemTime::now(),
        }
    }
}

impl DiscordClient {
    pub fn update(&mut self, detail_text: String, project_name: Option<String>) {
        let discord = self.discord.clone();
        let start_time = self.start_time.clone();
        // We do this async to avoid blocking the main thread.
        let promise = self
            .activity_promise
            .get_or_insert(poll_promise::Promise::spawn_async(async move {
                // Create the activity.
                let activity = discord_sdk::activity::ActivityBuilder::default()
                    .details(detail_text)
                    .state(
                        project_name
                            .map_or("No project open".to_string(), |n| format!("Editing {}", n)),
                    )
                    .assets(discord_sdk::activity::Assets::default().large(
                        "icon-1024".to_string(),
                        Some("https://luminol.dev".to_string()),
                    ))
                    .start_timestamp(start_time);

                // Update the activity.
                let _ = discord.update_activity(activity).await;
                // Return the Instant we finished.
                Instant::now()
            }));

        if let Some(instant) = promise.ready() {
            // Don't over-ping the API. It has a limit of 5 requests every 20 seconds.
            if instant.elapsed() > Duration::from_secs(4) {
                self.activity_promise = None;
            }
        }
    }
}
