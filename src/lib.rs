//! License checking for uv-resolved Python dependencies.
//!
//! The library accepts data and has no filesystem, network, environment, or clock access.

pub mod evidence;
pub mod inspection;
pub mod model;
pub mod policy;
pub mod report;
pub mod resolution;
