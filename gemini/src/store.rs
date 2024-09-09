use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use gemini_api::{body::GenerationConfig, model::blocking::Gemini};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct StoreData {
    pub key: String,
    pub url: String,
    pub options: GenerationConfig,
}

/// 配置文件名
const CONFIG_FILE_NAME: &str = "gemini.json";

/// 保存配置
pub fn save_config(gemini: Gemini) -> Result<()> {
    let data = StoreData {
        key: gemini.key.clone(),
        url: gemini.url.clone(),
        options: gemini.options.clone(),
    };
    let json_data = serde_json::to_string(&data).unwrap();
    let config_file = get_config_file()?;
    let mut file = File::create(config_file)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

/// 读取配置
pub fn read_config() -> Result<Gemini> {
    let config_file = get_config_file()?;
    if config_file.exists() {
        let mut file = File::open(config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let data: StoreData = serde_json::from_str(&contents)?;
        let gemini = Gemini::rebuild(data.key, data.url, Vec::new(), data.options);
        Ok(gemini)
    } else {
        bail!("配置文件不存在")
    }
}

/// 获取配置文件路径
fn get_config_file() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    Ok(exe_path.parent().unwrap().join(CONFIG_FILE_NAME))
}
