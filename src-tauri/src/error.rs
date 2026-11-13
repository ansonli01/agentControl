use serde::{Serialize, Serializer};

/// 统一错误类型：前端可直接展示 `message`。
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),

    #[error("未找到智能体：{0}")]
    AgentNotFound(String),

    #[error("配置项缺失：{0}")]
    MissingConfig(String),

    #[error("命令执行失败（{code}）：{stderr}")]
    CommandFailed { code: i32, stderr: String },

    #[error("IO 错误：{0}")]
    Io(#[from] std::io::Error),

    #[error("序列化错误：{0}")]
    Json(#[from] serde_json::Error),

    #[error("数据库错误：{0}")]
    Db(#[from] rusqlite::Error),

    #[error("凭据库错误：{0}")]
    Keyring(String),

    #[error("网络错误：{0}")]
    Http(String),
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::Http(value.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(value: anyhow::Error) -> Self {
        AppError::Message(value.to_string())
    }
}

impl AppError {
    pub fn msg<S: Into<String>>(s: S) -> Self {
        AppError::Message(s.into())
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
