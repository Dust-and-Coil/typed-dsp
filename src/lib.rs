//! Strongly typed audio units: signal levels and gains, linear and in
//! decibels, that can't hold NaN.

#![no_std]

mod units;

pub use units::{GainDb, LevelDbfs, LinearGain, PeakAmplitude};
