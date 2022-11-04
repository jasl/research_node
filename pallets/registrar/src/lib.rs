#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits;
pub mod primitives;

mod pallet;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
