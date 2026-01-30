//! # CalliGlyph
//!
//! <p align="center">A lightweight terminal text editor built in Rust — minimal, fast, and designed for clarity.</p>
//!
//! ## Overview
//!
//! **CalliGlyph** (from *calligraphy* + *glyph*) is a simple, minimalistic **terminal-based text editor** written in **Rust**.
//! It focuses on speed, simplicity, and portability across Unix-like systems.
//!
//! ## Module Architecture
//!
//! * [`core`]: The central engine and state management.
//! * [`ui`]: Terminal rendering logic and TUI components.
//! * [`input`]: Keyboard and mouse event handling.
//! * [`plugins`]: Extensible components for customizing behavior.
//! * [`config`]:  Management of settings and persistent data.
//! * [`errors`]:  Management of custom error structs.

//expose modules
#[macro_use]
pub mod macros;

pub mod core;

pub mod config;

pub mod input;
pub mod ui;

pub mod errors;

pub mod app_config;
pub mod args;

/// This module is
pub mod plugins;
