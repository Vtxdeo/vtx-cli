use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// 项目配置结构体
/// 对应项目根目录下的 vtx.toml 文件
#[derive(Deserialize, Debug, Clone)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
}

/// 项目基础元数据与构建选项
#[derive(Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    /// 插件包名称，用于标识和产物命名
    pub name: String,

    /// 项目语言标识 (如: rust, go, ts, python, php, lua)
    /// 该字段决定了 CLI 采用何种构建策略
    pub language: String,

    /// 自定义构建命令
    /// 用于覆盖默认的构建逻辑，或为无标准工具链的语言(如 Lua)提供构建脚本。
    /// 示例: "composer run build" 或 "python build.py"
    pub build_cmd: Option<String>,

    /// 自定义输出目录
    /// 指定构建产物(.wasm)的存放位置。
    /// 若未指定，CLI 将在 dist, build, target 等标准目录中按既定策略搜索。
    pub output_dir: Option<String>,
}

/// 加载并解析当前目录下的 vtx.toml 配置文件
///
/// # 边界说明
/// - 必须在项目根目录调用，否则返回错误。
/// - 文件大小预期在 KB 级别，采用同步 IO 读取。
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
