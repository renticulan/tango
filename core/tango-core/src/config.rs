use std::io::{Read, Write};

use serde::Deserialize;

use crate::{i18n, input};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Theme {
    Light,
    Dark,
}

fn serialize_language_identifier<S>(
    v: &unic_langid::LanguageIdentifier,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&v.to_string())
}

fn deserialize_language_identifier<'de, D>(
    deserializer: D,
) -> Result<unic_langid::LanguageIdentifier, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    buf.parse().map_err(serde::de::Error::custom)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub nickname: Option<String>,
    pub theme: Theme,
    #[serde(
        serialize_with = "serialize_language_identifier",
        deserialize_with = "deserialize_language_identifier"
    )]
    pub language: unic_langid::LanguageIdentifier,
    pub max_queue_length: u32,
    pub input_mapping: input::Mapping,
    pub signaling_endpoint: String,
    pub iceconfig_endpoint: String,
    pub replaycollector_endpoint: String,
    pub patch_repo: String,
    pub default_input_delay: u32,
    pub default_match_type: u8,
    pub saves_path: std::path::PathBuf,
    pub roms_path: std::path::PathBuf,
    pub replays_path: std::path::PathBuf,
    pub patches_path: std::path::PathBuf,
}

fn get_config_path() -> Result<std::path::PathBuf, anyhow::Error> {
    Ok(
        directories_next::ProjectDirs::from("com.tangobattle", "", "Tango")
            .ok_or_else(|| anyhow::anyhow!("could not get tango project directory"))?
            .config_dir()
            .join("config.json"),
    )
}

impl Config {
    pub fn system_defaults() -> Result<Self, anyhow::Error> {
        let user_dirs = directories_next::UserDirs::new()
            .ok_or_else(|| anyhow::anyhow!("could not get user directories"))?;

        let tango_data_dir = user_dirs
            .document_dir()
            .ok_or_else(|| anyhow::anyhow!("could not get tango data directory"))?
            .join("Tango");

        Ok(Self {
            nickname: None,
            theme: Theme::Light,
            language: sys_locale::get_locale()
                .unwrap_or(i18n::FALLBACK_LANG.to_string())
                .parse()?,
            max_queue_length: 1200,
            input_mapping: input::Mapping::default(),
            signaling_endpoint: "".to_string(),
            iceconfig_endpoint: "".to_string(),
            replaycollector_endpoint: "https://replaycollector.tangobattle.com".to_string(),
            patch_repo: "".to_string(),
            default_input_delay: 2,
            default_match_type: 1,
            saves_path: tango_data_dir.join("saves"),
            roms_path: tango_data_dir.join("roms"),
            replays_path: tango_data_dir.join("replays"),
            patches_path: tango_data_dir.join("patches"),
        })
    }

    pub fn load_or_create() -> Result<Self, anyhow::Error> {
        let config_path = get_config_path()?;
        match std::fs::File::open(&config_path) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let config = serde_json::from_str(&contents)?;
                Ok(config)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                let config = Self::system_defaults()?;
                let contents = serde_json::to_string(&config)?;
                std::fs::create_dir_all(config_path.parent().unwrap())?;
                std::fs::File::create(&config_path)?.write_all(contents.as_bytes())?;
                Ok(config)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        let contents = serde_json::to_string(self)?;
        let mut file = std::fs::File::create(get_config_path()?)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn ensure_dirs(&self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.saves_path)?;
        std::fs::create_dir_all(&self.roms_path)?;
        std::fs::create_dir_all(&self.replays_path)?;
        std::fs::create_dir_all(&self.patches_path)?;
        Ok(())
    }
}
