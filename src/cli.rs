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
        /// 工作区包名，例如：vtx-plugin-auth-basic
        #[arg(short, long)]
        package: String,

        /// 构建目标，默认值为 "wasm32-wasip1"
        #[arg(long, default_value = "wasm32-wasip1")]
        target: String,

        /// 是否启用 release 模式，默认启用
        #[arg(long, default_value_t = true)]
        release: bool,
    },
}
