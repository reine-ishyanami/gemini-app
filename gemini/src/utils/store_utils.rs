use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use gemini_api::{body::request::GenerationConfig, param::LanguageModel};
use serde::{Deserialize, Serialize};

use super::db_utils::current_db_version;

/// 存储配置数据
#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct StoreData {
    pub key: String,
    pub model: LanguageModel,
    pub system_instruction: Option<String>,
    pub options: GenerationConfig,
    pub db_version: Option<String>,
}

/// 配置文件名
const CONFIG_FILE_NAME: &str = "gemini.json";

/// 保存配置
pub(crate) fn save_config(store_data: StoreData) -> Result<()> {
    let json_data = serde_json::to_string(&store_data).unwrap();
    let config_file = get_config_file()?;
    let mut file = File::create(config_file)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

/// 保存数据库版本变更
pub(crate) fn update_db_version_into_profile() -> Result<()> {
    let mut config = read_config()?;
    config.db_version = Some(current_db_version());
    save_config(config)
}

/// 读取配置
pub(crate) fn read_config() -> Result<StoreData> {
    let config_file = get_config_file()?;
    if config_file.exists() {
        let mut file = File::open(config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str::<StoreData>(&contents)?)
    } else {
        bail!("配置文件不存在")
    }
}

/// 获取配置文件路径
fn get_config_file() -> Result<PathBuf> {
    let exe_path = env::current_exe()?;
    Ok(exe_path.parent().unwrap().join(CONFIG_FILE_NAME))
}
