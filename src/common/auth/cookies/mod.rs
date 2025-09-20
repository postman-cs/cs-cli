//! Cookie management modules
//!
//! Handles cookie retrieval from authenticated browser sessions,
//! cookie storage strategies, and hybrid cookie synchronization.

pub mod cookie_retriever;
pub mod cookie_retrieval_trait;
pub mod guided_cookie_storage;
pub mod hybrid_cookie_storage;

pub use cookie_retriever::*;
pub use cookie_retrieval_trait::*;
pub use guided_cookie_storage::*;
pub use hybrid_cookie_storage::*;