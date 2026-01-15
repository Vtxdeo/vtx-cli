use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 项目配置结构体
/// 对应项目根目录下的 vtx.toml 文件
#[derive(Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    pub build: Option<BuildConfig>,
    pub sdk: Option<SdkConfig>,
}

/// 项目作者信息
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
}

/// 项目基础元数据
#[derive(Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    /// 插件包名称，用于标识和产物命名
    pub name: String,

    /// 项目语言标识 (如 rust, go, ts, python, php, lua)
    /// 该字段决定了 CLI 采用何种构建策略
    pub language: String,

    /// 插件作者（用于写入 .vtx 元数据，便于仓库与安全扫描）
    pub author: Option<String>,

    /// 作者列表（PEP 621 风格）
    pub authors: Option<Vec<ProjectAuthor>>,

    /// 项目描述
    pub description: Option<String>,

    /// 许可证标识
    pub license: Option<String>,

    /// 主页链接
    pub homepage: Option<String>,

    /// 仓库地址
    pub repository: Option<String>,

    /// 关键词
    pub keywords: Option<Vec<String>>,
}

/// 构建配置
#[derive(Deserialize, Debug, Clone)]
pub struct BuildConfig {
    /// 自定义构建命令
    /// 用于覆盖默认的构建逻辑
    pub cmd: Option<String>,

    /// 自定义输出目录
    /// 指定构建产物(.wasm)的存放位置
    pub output_dir: Option<String>,

    /// 精确的产物文件名
    pub artifact: Option<String>,
}

/// SDK 配置
#[derive(Deserialize, Debug, Clone)]
pub struct SdkConfig {
    /// SDK 版本
    pub version: Option<String>,
}

/// 加载并解析当前目录下的 vtx.toml 配置文件
///
/// # 边界说明
/// - 必须在项目根目录调用，否则返回错误
/// - 文件大小预期在 KB 级别，采用同步 IO 读取
pub fn load() -> Result<ProjectConfig> {
    let config_path = Path::new("vtx.toml");

    if !config_path.exists() {
        anyhow::bail!("Configuration file 'vtx.toml' not found in current directory.");
    }

    let content = fs::read_to_string(config_path).context("Failed to read vtx.toml file")?;

    let config: ProjectConfig =
        toml::from_str(&content).context("Failed to parse vtx.toml content")?;

    Ok(config)
}
