// Integration tests entry point
//
// Test fixtures and helper modules naturally accumulate functions that are
// only used by a subset of tests; allow dead-code and unused-import lints
// crate-wide for the integration test binary.
#![allow(dead_code, unused_imports, unused_variables)]

mod integration;
