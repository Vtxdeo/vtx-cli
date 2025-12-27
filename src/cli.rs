use clap::{Parser, Subcommand};

/// 主程序的命令行接口（CLI）结构体
/// 用于解析命令行参数并提供命令的处理
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// 子命令部分，包含不同的命令类型
    #[command(subcommand)]
    pub command: Commands,
}

/// 所有支持的子命令
#[derive(Subcommand)]
pub enum Commands {
    /// 构建并打包插件（wasm -> component -> .vtx）
    Build {
        /// 工作区包名。如果未指定，将尝试从 vtx.toml 配置文件中读取。
        #[arg(short, long)]
        package: Option<String>,

        /// 构建目标，默认值为 "wasm32-wasip1"
        #[arg(long, default_value = "wasm32-wasip1")]
        target: String,

        /// 是否启用 release 模式，默认启用
        #[arg(long, default_value_t = true)]
        release: bool,

        /// 强制模式：忽略 SDK 版本不匹配或非致命的契约检查错误
        #[arg(long, default_value_t = false)]
        force: bool,

        /// 调试模式：保留调试符号，输出详细的构建与检查日志
        #[arg(long, default_value_t = false)]
        debug: bool,
    },
}
