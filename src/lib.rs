#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod debruijn;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod lambda;
pub mod parsing;
pub mod r#macro;
pub mod stdlib;
pub mod strategy;
pub mod typing;

#[cfg(test)]
mod tests;
