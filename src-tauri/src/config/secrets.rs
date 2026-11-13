use crate::error::{AppError, AppResult};

/// 凭据服务名（Windows 凭据管理器中的目标名）。
const SERVICE: &str = "AgentControl";

fn entry(id: &str) -> AppResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, id).map_err(|e| AppError::Keyring(e.to_string()))
}

/// 写入密钥到系统凭据库（Windows 下由 DPAPI 加密）。
pub fn set_secret(id: &str, secret: &str) -> AppResult<()> {
    entry(id)?
        .set_password(secret)
        .map_err(|e| AppError::Keyring(e.to_string()))
}

/// 读取密钥。
pub fn get_secret(id: &str) -> AppResult<Option<String>> {
    match entry(id)?.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

/// 删除密钥。
pub fn delete_secret(id: &str) -> AppResult<()> {
    match entry(id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keyring(e.to_string())),
    }
}

/// 掩码显示：保留前 3 位与后 4 位。
pub fn mask(secret: &str) -> String {
    let chars: Vec<char> = secret.chars().collect();
    if chars.len() <= 8 {
        return "********".into();
    }
    let head: String = chars[..3].iter().collect();
    let tail: String = chars[chars.len() - 4..].iter().collect();
    format!("{head}****{tail}")
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_keeps_head_and_tail() {
        assert_eq!(mask("sk-abcdefghijkl"), "sk-****ijkl");
    }

    #[test]
    fn mask_short_secret() {
        assert_eq!(mask("short"), "********");
    }
}