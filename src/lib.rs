//! Exact seven-card poker hand evaluation using an embedded canonical rank table.
//!
//! [`evaluate_7`] validates that all seven cards are distinct. Callers that
//! already guarantee uniqueness can use [`evaluate_7_unchecked`] to skip that
//! application-level check. Both APIs return the same directly comparable
//! [`HandRank`] for valid hands.
//!
//! The evaluator constructs four 13-bit suit masks, canonicalises suit order,
//! maps the resulting key to a dense formula index, and reads a `u16` rank from
//! an embedded table. It performs no runtime filesystem access.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "bench-internals")]
#[doc(hidden)]
pub mod benchmark_support;
mod canonical;
mod card;
#[cfg(test)]
mod direct;
mod evaluator;
mod formula;
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
pub use rank::{HandCategory, HandRank};
