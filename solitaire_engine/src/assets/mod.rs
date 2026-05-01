//! Asset-loading infrastructure for runtime SVG rasterisation.
//!
//! See `CARD_PLAN.md` for the multi-phase implementation plan. This module
//! is the entry point for Phase 1 (the SVG → `Image` asset loader). Later
//! phases extend it with custom asset sources for embedded and user
//! themes, and a `CardTheme` asset that aggregates 53 image handles.

pub mod svg_loader;

pub use svg_loader::{rasterize_svg, SvgLoader, SvgLoaderError, SvgLoaderSettings};
