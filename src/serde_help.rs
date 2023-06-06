use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use directories::ProjectDirs;

use iced::widget::image;

use ::image as img;

use serde::{Deserialize, Deserializer};

use crate::tft_data::Handle;

const CDRAGON_URL: &str = "https://raw.communitydragon.org/latest/game/";

static DIR: OnceLock<ProjectDirs> = OnceLock::new();
static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn deserialize_image<'de, D>(deserializer: D) -> Result<Handle, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    if opt.is_none() {
        return Ok(Handle::default());
    }
    let s = opt
        .unwrap()
        .to_lowercase() // url needs to be lowercase
        .replace("dds", "png") // replace dds file with png
        .replace("tex", "png");
    let mut url = String::from(CDRAGON_URL);
    url.push_str(&s);

    let path: Vec<&str> = url.split('/').collect();
    let file_name = path.last().unwrap();

    let cache_dir = CACHE_DIR.get().unwrap();

    let cache_path = cache_dir.join(file_name);

    let image = if !Path::exists(&cache_dir.join(path.last().unwrap())) {
        let mut buf: Vec<u8> = vec![];
        ureq::get(&url)
            .call()
            .unwrap()
            .into_reader()
            .read_to_end(&mut buf)
            .unwrap();

        let img_mem = img::load_from_memory(&buf).unwrap();

        // dbg!(img_mem.height(), img_mem.width());
        // dbg!(&url);

        let img_mem = if img_mem.width() > 128 {
            img_mem.resize(128, 128, img::imageops::FilterType::CatmullRom)
        } else {
            img_mem
        };

        // dbg!(img_mem.height(), img_mem.width());

        img_mem.save(cache_path).unwrap();
        // fs::write(cache_path, &img_mem).unwrap();

        println!("{} has been cached", file_name);
        image::Handle::from_memory(img_mem.as_bytes().to_owned())
    } else {
        image::Handle::from_path(cache_path)
    };

    Ok(Handle { handle: image, url })
}

pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
