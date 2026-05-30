//! Chainforge EVM 兼容执行层。

pub mod executor;
pub mod state;

pub use executor::EvmExecutor;
pub use state::InMemoryEvmState;

// Re-export revm primitives for downstream crates (RPC, etc.)
pub use revm::primitives::{Address, U256};
