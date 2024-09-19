use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::{bail, Result};
use gemini_api::{body::request::GenerationConfig, param::LanguageModel};
use serde::{Deserialize, Serialize};

/// 存储配置数据
#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct StoreData {
    pub key: String,
    pub model: LanguageModel,
    pub system_instruction: Option<String>,
    pub options: GenerationConfig,
}

/// 配置文件名
const CONFIG_FILE_NAME: &str = "gemini.json";

/// 保存配置
pub fn save_config(gemini: StoreData) -> Result<()> {
    let json_data = serde_json::to_string(&gemini).unwrap();
    let config_file = get_config_file()?;
    let mut file = File::create(config_file)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

/// 读取配置
pub fn read_config() -> Result<StoreData> {
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
