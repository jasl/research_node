
//! Autogenerated weights for pallet_computing_workers
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-22, STEPS: `50`, REPEAT: `50`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! CPU: `<UNKNOWN>`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
    // target/production/node
    // benchmark
    // pallet
    // --pallet=pallet_computing_workers
    // --extrinsic=*
    // --chain=dev
    // --steps=50
    // --repeat=50
    // --execution=wasm
    // --wasm-execution=compiled
    // --heap-pages=4096
    // --output=./pallets/computing_workers/src/weights.rs
    // --template=./templates/pallet-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use sp_std::marker::PhantomData;

/// Weight functions needed for pallet_computing_workers.
pub trait WeightInfo {
    fn register() -> Weight;
    fn deregister() -> Weight;
    fn deposit() -> Weight;
    fn withdraw() -> Weight;
    fn online() -> Weight;
    fn refresh_attestation() -> Weight;
    fn request_offline() -> Weight;
    fn request_offline_for() -> Weight;
    fn force_offline() -> Weight;
    fn force_offline_for() -> Weight;
    fn heartbeat() -> Weight;
}

/// Weights for pallet_computing_workers using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
        impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForWorkers (r:1 w:1)
            /// Proof: ComputingWorkers CounterForWorkers (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn register() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `966`
        //  Estimated: `5704`
        // Minimum execution time: 34_000 nanoseconds.
        Weight::from_ref_time(35_000_000)
        .saturating_add(Weight::from_proof_size(5704))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForWorkers (r:1 w:1)
            /// Proof: ComputingWorkers CounterForWorkers (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn deregister() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1343`
        //  Estimated: `5704`
        // Minimum execution time: 35_000 nanoseconds.
        Weight::from_ref_time(37_000_000)
        .saturating_add(Weight::from_proof_size(5704))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:0)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
        fn deposit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1343`
        //  Estimated: `5205`
        // Minimum execution time: 23_000 nanoseconds.
        Weight::from_ref_time(23_000_000)
        .saturating_add(Weight::from_proof_size(5205))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:0)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
        fn withdraw() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1343`
        //  Estimated: `5205`
        // Minimum execution time: 23_000 nanoseconds.
        Weight::from_ref_time(23_000_000)
        .saturating_add(Weight::from_proof_size(5205))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:0)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: Timestamp Now (r:1 w:0)
            /// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CurrentFlipFlopStartedAt (r:1 w:0)
            /// Proof: ComputingWorkers CurrentFlipFlopStartedAt (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
            /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
            /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipOrFlop (r:1 w:0)
            /// Proof: ComputingWorkers FlipOrFlop (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn online() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1257`
        //  Estimated: `12802`
        // Minimum execution time: 64_000 nanoseconds.
        Weight::from_ref_time(66_000_000)
        .saturating_add(Weight::from_proof_size(12802))
            .saturating_add(T::DbWeight::get().reads(8_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: Timestamp Now (r:1 w:0)
            /// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
        fn refresh_attestation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `945`
        //  Estimated: `3105`
        // Minimum execution time: 55_000 nanoseconds.
        Weight::from_ref_time(56_000_000)
        .saturating_add(Weight::from_proof_size(3105))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn request_offline() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn request_offline_for() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn force_offline() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn force_offline_for() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:0)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:0)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CurrentFlipFlopStartedAt (r:1 w:0)
            /// Proof: ComputingWorkers CurrentFlipFlopStartedAt (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
            /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
            /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipOrFlop (r:1 w:0)
            /// Proof: ComputingWorkers FlipOrFlop (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlipSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlipSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn heartbeat() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1352`
        //  Estimated: `15309`
        // Minimum execution time: 29_000 nanoseconds.
        Weight::from_ref_time(29_000_000)
        .saturating_add(Weight::from_proof_size(15309))
            .saturating_add(T::DbWeight::get().reads(9_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
        }
    }

    // For backwards compatibility and tests
    impl WeightInfo for () {
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForWorkers (r:1 w:1)
            /// Proof: ComputingWorkers CounterForWorkers (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn register() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `966`
        //  Estimated: `5704`
        // Minimum execution time: 34_000 nanoseconds.
        Weight::from_ref_time(35_000_000)
        .saturating_add(Weight::from_proof_size(5704))
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForWorkers (r:1 w:1)
            /// Proof: ComputingWorkers CounterForWorkers (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn deregister() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1343`
        //  Estimated: `5704`
        // Minimum execution time: 35_000 nanoseconds.
        Weight::from_ref_time(37_000_000)
        .saturating_add(Weight::from_proof_size(5704))
            .saturating_add(RocksDbWeight::get().reads(3_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:0)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
        fn deposit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1343`
        //  Estimated: `5205`
        // Minimum execution time: 23_000 nanoseconds.
        Weight::from_ref_time(23_000_000)
        .saturating_add(Weight::from_proof_size(5205))
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:0)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:1)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
        fn withdraw() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1343`
        //  Estimated: `5205`
        // Minimum execution time: 23_000 nanoseconds.
        Weight::from_ref_time(23_000_000)
        .saturating_add(Weight::from_proof_size(5205))
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:0)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: Timestamp Now (r:1 w:0)
            /// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CurrentFlipFlopStartedAt (r:1 w:0)
            /// Proof: ComputingWorkers CurrentFlipFlopStartedAt (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
            /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
            /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipOrFlop (r:1 w:0)
            /// Proof: ComputingWorkers FlipOrFlop (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn online() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1257`
        //  Estimated: `12802`
        // Minimum execution time: 64_000 nanoseconds.
        Weight::from_ref_time(66_000_000)
        .saturating_add(Weight::from_proof_size(12802))
            .saturating_add(RocksDbWeight::get().reads(8_u64))
            .saturating_add(RocksDbWeight::get().writes(3_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: Timestamp Now (r:1 w:0)
            /// Proof: Timestamp Now (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
        fn refresh_attestation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `945`
        //  Estimated: `3105`
        // Minimum execution time: 55_000 nanoseconds.
        Weight::from_ref_time(56_000_000)
        .saturating_add(Weight::from_proof_size(3105))
            .saturating_add(RocksDbWeight::get().reads(2_u64))
            .saturating_add(RocksDbWeight::get().writes(1_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn request_offline() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn request_offline_for() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn force_offline() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:1)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn force_offline_for() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `963`
        //  Estimated: `8123`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_ref_time(21_000_000)
        .saturating_add(Weight::from_proof_size(8123))
            .saturating_add(RocksDbWeight::get().reads(4_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
        }
            /// Storage: ComputingWorkers Workers (r:1 w:0)
            /// Proof: ComputingWorkers Workers (max_values: None, max_size: Some(127), added: 2602, mode: MaxEncodedLen)
            /// Storage: System Account (r:1 w:0)
            /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CurrentFlipFlopStartedAt (r:1 w:0)
            /// Proof: ComputingWorkers CurrentFlipFlopStartedAt (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
            /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
            /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipOrFlop (r:1 w:0)
            /// Proof: ComputingWorkers FlipOrFlop (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlopSet (r:1 w:1)
            /// Proof: ComputingWorkers FlopSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlopSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlopSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers FlipSet (r:1 w:1)
            /// Proof: ComputingWorkers FlipSet (max_values: None, max_size: Some(36), added: 2511, mode: MaxEncodedLen)
            /// Storage: ComputingWorkers CounterForFlipSet (r:1 w:1)
            /// Proof: ComputingWorkers CounterForFlipSet (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
        fn heartbeat() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1352`
        //  Estimated: `15309`
        // Minimum execution time: 29_000 nanoseconds.
        Weight::from_ref_time(29_000_000)
        .saturating_add(Weight::from_proof_size(15309))
            .saturating_add(RocksDbWeight::get().reads(9_u64))
            .saturating_add(RocksDbWeight::get().writes(4_u64))
        }
    }
