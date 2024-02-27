use std::time::{Duration, Instant};
use chrono::Local;
use rand::Rng;
use serde_json::Value;

#[derive(Debug)]
pub struct PhotoManager {
    pub photos: Vec<Photo>,
    pub currrent_img: usize,
    interval: Duration,
    refresh_interval: Duration,
    last_refresh: Instant,
    last_change: Instant,
    rng: rand::rngs::ThreadRng
}

impl PhotoManager {
    pub fn new(interval: Duration, refresh_interval: Duration) -> PhotoManager {
        let mut mgr = PhotoManager {
            photos: Vec::new(),
            currrent_img: 0,
            interval,
            refresh_interval,
            last_refresh: Instant::now(),
            last_change: Instant::now(),
            rng: rand::thread_rng()
        };
        mgr.refresh();
        mgr.next();
        mgr
    }

    pub fn refresh(&mut self) {
        let WHITELISTED_CAMS = [
            //"MCZ_RIGHT",
            //"MCZ_LEFT",
            "FRONT_HAZCAM_LEFT_A",
            "FRONT_HAZCAM_RIGHT_A",
            "REAR_HAZCAM_LEFT",
            "REAR_HAZCAM_RIGHT",
            //"SHERLOC_WATSON",
        ];
        let client = reqwest::blocking::Client::new();
        let now = Local::now();
        let mut offset = 0;

        let mut photos_found = false;

        while !photos_found {
            let yesterday = now - chrono::Duration::days(offset);
            let formatted_yesterday = yesterday.format("%Y-%m-%d").to_string();
    
            let url = format!("https://api.nasa.gov/mars-photos/api/v1/rovers/perseverance/photos?earth_date={}&api_key=LC2gWILgIzODdyY0KFH6yM7a6pZvKVhONm9iXfFi", formatted_yesterday).to_string();
            let response = client.get(url).send().unwrap();
            let json: serde_json::Value = response.json().unwrap();
            if !json["photos"].is_null() && json["photos"].as_array().unwrap().len() > 0 {
                let photos = json["photos"].as_array().unwrap().clone();

                let whitelisted_photos = photos.iter().filter(|photo| {
                    let camera = photo["camera"]["name"].as_str().unwrap();
                    WHITELISTED_CAMS.contains(&camera)
                }).map(|photo| {
                    Photo {
                        camera: photo["camera"]["name"].as_str().unwrap().to_string(),
                        src: photo["img_src"].as_str().unwrap().to_string(),
                        name: "Perseverance ".to_string() + photo["camera"]["name"].as_str().unwrap()
                    }
                }).collect::<Vec<Photo>>();

                if whitelisted_photos.len() > 0 {
                    photos_found = true;
                    self.photos = whitelisted_photos;
                } else {
                    offset += 1;
                }
            } else {
                offset += 1;
            }

            let accepted_extensions = ["jpg", "jpeg", "png"];
            let last_week = now - chrono::Duration::days(7);
            let formatted_today: String = now.format("%Y-%m-%d").to_string();
            let formatted_last_week: String = last_week.format("%Y-%m-%d").to_string();
            let url = format!("https://api.nasa.gov/planetary/apod?start_date={}&end_date={}&api_key=LC2gWILgIzODdyY0KFH6yM7a6pZvKVhONm9iXfFi", formatted_last_week, formatted_today);
            let response = client.get(url).send().unwrap();
            let json: serde_json::Value = response.json().unwrap();
            let photos = json.as_array().unwrap();
            let apod_photos = photos.iter().map(|photo| {
                let has_hd = photo["hdurl"].is_string();
                Photo {
                    camera: "APOD".to_string(),
                    src: if has_hd { photo["hdurl"].as_str().unwrap().to_string() } else { photo["url"].as_str().unwrap().to_string() },
                    name: photo["title"].as_str().unwrap().to_string()
                }
            }).filter(|photo| {
                let extension = photo.src.split(".").last().unwrap();
                accepted_extensions.contains(&extension)
            }).collect::<Vec<Photo>>();

            self.photos.extend(apod_photos);

            let url = "https://images-api.nasa.gov/search?q=''&year_start=2024&year_end=2024&media_type=image".to_string();
            let response = client.get(url).send().unwrap();
            let json: serde_json::Value = response.json().unwrap();
            let photo_items = json["collection"]["items"].as_array().unwrap();
            let photo_src_urls = photo_items.iter().map(|item| {
                item["href"].to_string()
            }).collect::<Vec<String>>();

            let nasa_photos = photo_src_urls.iter().map(|url| {
                self.get_nasa_image_from_json(&url.to_string())
            }).collect::<Vec<Photo>>();

            self.photos.extend(nasa_photos);
        }
    }

    pub fn get_nasa_image_from_json(&self, url: &String) -> Photo {
        let client = reqwest::blocking::Client::new();
        let mut url = url.clone().split_off(1);
        url = url.replace("\"", "");
        let response = client.get(url).send().unwrap();
        let json: serde_json::Value = response.json().unwrap();
        let photo_url = json.as_array().unwrap()[0].clone().to_string();
        Photo {
            camera: "NASA".to_string(),
            src: photo_url[1..photo_url.len()-1].to_string(),
            name: "".to_string()
        }
    }

    pub fn prev(&mut self) {
        self.currrent_img = (self.currrent_img + self.photos.len() - 1) % self.photos.len();
        let img = &self.photos[self.currrent_img];
        wallpaper::set_from_url(&img.src).unwrap();
        wallpaper::set_mode(wallpaper::Mode::Crop).unwrap();
        self.last_change = Instant::now();
    }

    pub fn next(&mut self) {
        if self.photos.len() == 0 {
            return;
        }

        let shift = (self.rng.gen::<f64>() * self.photos.len() as f64) as usize;
        self.currrent_img = (self.currrent_img + shift) % self.photos.len();
        let img = &self.photos[self.currrent_img];
        wallpaper::set_from_url(&img.src).unwrap();
        wallpaper::set_mode(wallpaper::Mode::Crop).unwrap();
        self.last_change = Instant::now();
    }


    pub fn next_if_ready(&mut self) {
        if self.last_change.elapsed() > self.interval {
            self.next();
        }
    }

    pub fn refresh_if_ready(&mut self) {
        if self.last_refresh.elapsed() > self.refresh_interval {
            self.refresh();
            self.last_refresh = Instant::now();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Photo {
    camera: String,
    src: String,
    pub name: String
}