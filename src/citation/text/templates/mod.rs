//! Citation style template modules.
//!
//! Each module implements one or more citation styles. Every module exports
//! `render_reference()` and `render_in_text()` functions with identical
//! signatures so the engine dispatcher can call them uniformly.

pub mod apa;
pub mod acs;
pub mod ama;
pub mod nature;
pub mod ieee;
pub mod vancouver;
pub mod apsa_asa;
pub mod chicago;
pub mod harvard;
pub mod mhra;
pub mod mla;
