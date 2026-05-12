use crate::base::config::Config;
use crate::kv::bot::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Admin,
    Backup,
}

/// Validate token and return its type if valid
pub fn validate_token(token: &str) -> Option<TokenType> {
    let config = Config::get();

    if let Some(admin_tokens) = &config.tokens {
        // Check admin tokens
        if admin_tokens.admin_tokens.contains(&token.to_string()) {
            return Some(TokenType::Admin);
        }

        // Check backup tokens
        if admin_tokens.backup_tokens.contains(&token.to_string()) {
            return Some(TokenType::Backup);
        }
    }

    None
}

/// Get permissions for a token type
pub fn get_perms_for_token(token_type: TokenType) -> Vec<Permission> {
    match token_type {
        TokenType::Admin => vec![Permission::Log, Permission::Status],
        TokenType::Backup => vec![Permission::Log, Permission::Status, Permission::Backup],
    }
}
