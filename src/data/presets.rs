use crate::data::LSystemParameters;
use lsystems_core::drawing::*;
use serde_json::*;
use std::include_str;

pub const KOCH_SNOWFLAKE: &'static str = include_str!("presets/koch.json");
pub const PENROSE: &'static str = include_str!("presets/penrose.json");