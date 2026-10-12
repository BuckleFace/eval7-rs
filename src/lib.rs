//! Seven-card poker hand evaluation using an embedded canonical rank table.

#![forbid(unsafe_code)]

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
