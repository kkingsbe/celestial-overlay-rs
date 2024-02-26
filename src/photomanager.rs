use std::time::{Duration, Instant};
use rand::Rng;

#[derive(Debug)]
pub struct PhotoManager {
    photos: Vec<Photo>,
    currrent_img: usize,
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
            "MCZ_RIGHT",
            "MCZ_LEFT",
            "FRONT_HAZCAM_LEFT_A",
            "FRONT_HAZCAM_RIGHT_A",
            "REAR_HAZCAM_LEFT",
            "REAR_HAZCAM_RIGHT",
            "SHERLOC_WATSON",
        ];

        let client = reqwest::blocking::Client::new();
        let response = client.get("https://api.nasa.gov/mars-photos/api/v1/rovers/perseverance/photos?earth_date=2024-02-23&api_key=DEMO_KEY").send().unwrap();
        let json: serde_json::Value = response.json().unwrap();
        let photos = json["photos"].as_array().unwrap();
        let whitelisted_photos = photos.iter().filter(|photo| {
            let camera = photo["camera"]["name"].as_str().unwrap();
            WHITELISTED_CAMS.contains(&camera)
        }).map(|photo| {
            Photo {
                camera: photo["camera"]["name"].as_str().unwrap().to_string(),
                src: photo["img_src"].as_str().unwrap().to_string()
            }
        }).collect::<Vec<Photo>>();

        self.photos = whitelisted_photos;
    }

    pub fn prev(&mut self) {
        self.currrent_img = (self.currrent_img + self.photos.len() - 1) % self.photos.len();
        let img = &self.photos[self.currrent_img];
        wallpaper::set_from_url(&img.src).unwrap();
        wallpaper::set_mode(wallpaper::Mode::Crop).unwrap();
        self.last_change = Instant::now();
    }

    pub fn next(&mut self) {
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

#[derive(Debug)]
pub struct Photo {
    camera: String,
    src: String
}