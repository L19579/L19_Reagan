pub mod curve;
pub mod constraints; 
pub mod entrypoint;
pub mod instruction; 
pub mod processor; 
pub mod state;
pub mod error;

//Probably won't need most of this section.
#[cfg(not(feature = "no-entrypoint"))]
// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
solana_program::declare_id!("SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw");
