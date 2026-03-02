//! Artemis Core - 核心数据模型和协议定义
//!
//! 这个 crate 包含 Artemis 的核心数据结构和协议定义，
//! 可被 client 和 server 共同使用。

pub mod error;
pub mod model;

// 重新导出常用类型
pub use error::ArtemisError;
pub use model::*;
