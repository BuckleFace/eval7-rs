//! Exact seven-card poker hand evaluation using embedded compact rank tables.
//!
//! [`evaluate_7`] validates that all seven cards are distinct. Callers that
//! already guarantee uniqueness can use [`evaluate_7_unchecked`] to skip that
//! application-level check. Both APIs return the same directly comparable
//! [`HandRank`] for valid hands.
//! [`PreparedBoard`] validates a five-card board once for repeated evaluation
//! with different two-card hands.
//!
//! The evaluator aggregates rank multiplicities and suit masks. Flushes use a
//! direct suit-mask lookup; other hands use a dense combinatorial rank-count
//! index. All support data is static, with no runtime allocation or file access.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub mod benchmark_support;
#[cfg(feature = "bench-internals")]
mod canonical;
mod card;
#[cfg(test)]
mod direct;
mod evaluator;
mod hybrid;
mod index;
mod prepared;
mod rank;
#[cfg(test)]
mod reference;
#[cfg(test)]
mod state;
mod table;
#[cfg(test)]
mod tables;
#[cfg(test)]
mod tests;

pub use card::{Card, CardError};
pub use evaluator::{EvaluationError, Evaluator, evaluate_7, evaluate_7_unchecked};
pub use prepared::PreparedBoard;
pub use rank::{HandCategory, HandRank};
