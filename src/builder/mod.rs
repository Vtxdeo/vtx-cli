use anyhow::Result;
use std::path::PathBuf;

pub mod go;
pub mod lua;
pub mod php;
pub mod python;
pub mod rust;
pub mod ts;

/// 构建流水线接口
///
/// 该 Trait 定义了将源码转换为中间态 Wasm 文件的标准生命周期。
/// 实现者应当遵循无状态原则，或明确标注对文件系统的副作用。
pub trait Builder {
    /// 阶段 1: 环境预检
    ///
    /// 验证宿主环境是否满足构建要求。
    ///
    /// # 行为说明
    /// - 应通过执行 `--version` 等轻量命令检查工具链是否存在。
    /// - 若检查失败，必须返回包含具体安装建议的 Error。
    fn check_env(&self) -> Result<()>;

    /// 阶段 2: 执行构建
    ///
    /// 调用底层工具链进行编译。
    ///
    /// # 参数
    /// - `package`: 包名，用于指定构建目标或传递给构建脚本。
    /// - `target`: 目标架构标识（如 wasm32-wasi）。
    /// - `release`: 构建模式，true 表示优化后的发布模式。
    ///
    /// # 副作用
    /// - 产生磁盘 IO，生成编译中间产物。
    /// - 可能消耗大量 CPU/内存资源。
    /// - 可能会向 stdout/stderr 写入底层工具链的日志。
    fn build(&self, package: &str, target: &str, release: bool) -> Result<()>;

    /// 阶段 3: 产物定位
    ///
    /// 在构建完成后，定位最终生成的 Wasm 文件路径。
    ///
    /// # 返回值
    /// - 成功：返回绝对路径或相对于执行目录的路径。
    /// - 失败：若找不到文件或存在歧义，返回 Error。
    fn find_output(&self, package: &str, target: &str, release: bool) -> Result<PathBuf>;
}
