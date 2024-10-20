#![cfg_attr(not(feature = "std"), no_std)]

pub mod debruijn;
pub mod environment;
pub mod error;
pub mod interpreter;
pub mod lambda;
pub mod parsing;
pub mod r#macro;
pub mod stdlib;
pub mod strategy;

#[cfg(test)]
mod tests;
