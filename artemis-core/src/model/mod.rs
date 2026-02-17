// 核心数据模型 - client/server 共享
pub mod change;
pub mod instance;
pub mod request;
pub mod service;

// 复制协议 - server 间通信
pub mod replication;

// 重新导出核心类型
pub use change::{ChangeType, InstanceChange};
pub use instance::{Instance, InstanceKey, InstanceStatus};
pub use request::*;
pub use service::Service;

// 重新导出复制协议类型
pub use replication::*;
