extern crate regex;
use egui::TextureHandle;
use regex::Regex;

use std::time::{Duration, Instant};

use chrono::Local;
use serde_json::Value;

pub struct SpaceWeatherManager {
    pub weather_report: Option<WeatherReport>,
    refresh_interval: Duration,
    last_refresh: Instant,
}

impl SpaceWeatherManager {
    pub fn new(refresh_interval: Duration) -> SpaceWeatherManager {
        let mut mgr = SpaceWeatherManager {
            weather_report: None,
            refresh_interval,
            last_refresh: Instant::now(),
        };
        mgr.refresh();
        mgr
    }

    pub fn refresh(&mut self) {
        let now = Local::now();
        let yesterday = now - chrono::Duration::days(1);
        let tomorrow = now + chrono::Duration::days(1);

        let formatted_yesterday = yesterday.format("%Y-%m-%d").to_string();
        let formatted_tomorrow = tomorrow.format("%Y-%m-%d").to_string();

        let client = reqwest::blocking::Client::new();
        let response = client.get(format!("https://api.nasa.gov/DONKI/notifications?startDate={}&endDate={}&type=all&api_key=LC2gWILgIzODdyY0KFH6yM7a6pZvKVhONm9iXfFi", formatted_yesterday, formatted_tomorrow)).send().unwrap();
        let json: serde_json::Value = response.json().unwrap();
        let messages: &Vec<serde_json::Value> = json.as_array().unwrap();

        self.weather_report = Some(WeatherReport::from_json(messages[0].clone()));
    }

    pub fn refresh_if_ready(&mut self) {
        if self.last_refresh.elapsed() > self.refresh_interval {
            self.refresh();
            self.last_refresh = Instant::now();
        }
    }
}

pub struct WeatherReport {
    pub message: String,
    pub message_type: String,
    pub time: String,
    pub images: Vec<egui::ColorImage>,
    pub active_image_handle: Option<TextureHandle>,
    pub active_image: usize,
    pub image_interval: Duration,
    pub last_image_change: Instant,
}

impl WeatherReport {
    pub fn from_json(json: Value) -> WeatherReport {
        let message = "## Summary:\n\n".to_string() + json["messageBody"].as_str().unwrap().to_string().split("Summary:\n\n").last().unwrap();
        let message_type = json["messageType"].as_str().unwrap().to_string();
        let time = json["messageIssueTime"].as_str().unwrap().to_string();
        let image_links = extract_image_links(message.as_str());

        WeatherReport {
            message: message.clone(),
            message_type,
            time,
            images: image_links.iter().map(|link| WeatherReport::load_image(link)).collect(),
            active_image_handle: None,
            active_image: 0,
            image_interval: Duration::from_secs(10),
            last_image_change: Instant::now(),
        }
    }

    fn load_image(url: &str) -> egui::ColorImage {
        let client = reqwest::blocking::Client::new();
        let response = client.get(url).send().unwrap();
        let bytes = response.bytes().unwrap();
        let image = image::load_from_memory(&bytes).unwrap().to_rgba8();
        let dimensions = image.dimensions();

        egui::ColorImage::from_rgba_unmultiplied([dimensions.0 as _, dimensions.1 as _], &image)
    }

    pub fn next_image(&mut self, egui_context: &egui::Context) {
        if self.images.len() == 0 {
            return;
        }
        
        self.active_image = (self.active_image + 1) % self.images.len();
        let image = &self.images[self.active_image];
        let texture_handle = egui_context.load_texture(&self.active_image.to_string(), image.clone(), egui::TextureOptions::LINEAR);
        self.active_image_handle = Some(texture_handle);
    }

    pub fn next_if_ready(&mut self, egui_context: &egui::Context) {
        if self.last_image_change.elapsed() > self.image_interval {
            self.next_image(egui_context);
            self.last_image_change = Instant::now();
        }
    }
}

fn extract_image_links(text: &str) -> Vec<String> {
    let image_url_pattern = r"https?://[\w/._-]+\.(?:jpg|jpeg|png|gif)";
    let re = Regex::new(image_url_pattern).unwrap();
    
    re.find_iter(text)
        .map(|mat| mat.as_str().to_string())
        .collect()
}