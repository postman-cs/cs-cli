//! Generic session management traits
//!
//! Provides common session handling patterns that can be implemented
//! by both Gong and Slack authentication managers.

use crate::Result;
use std::collections::HashMap;
use async_trait::async_trait;

/// Generic session management trait
#[async_trait]
pub trait SessionManager {
    type SessionData;
    
    /// Initialize a new session
    async fn initialize_session(&mut self) -> Result<()>;
    
    /// Authenticate and establish session
    async fn authenticate(&mut self) -> Result<bool>;
    
    /// Check if current session is valid
    async fn is_authenticated(&self) -> bool;
    
    /// Refresh session if needed
    async fn refresh_session(&mut self) -> Result<bool>;
    
    /// Get session data
    fn get_session_data(&self) -> Option<&Self::SessionData>;
    
    /// Get authentication headers
    fn get_auth_headers(&self) -> HashMap<String, String>;
}

/// Generic authentication token trait  
pub trait AuthToken {
    /// Check if token is expired
    fn is_expired(&self) -> bool;
    
    /// Get token value
    fn get_token(&self) -> &str;
    
    /// Get token type (e.g., "Bearer", "xoxc")
    fn get_token_type(&self) -> &str;
}

/// Generic session data that holds authentication information
pub trait SessionData {
    /// Get unique session identifier
    fn session_id(&self) -> &str;
    
    /// Get workspace/domain identifier  
    fn workspace_id(&self) -> &str;
    
    /// Check if session data is valid
    fn is_valid(&self) -> bool;
}
