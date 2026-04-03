# Changelog

## [0.1.0] - 2026-04-03

- Initialized the `tuner-libs` workspace and split shared code into dedicated crates.
- Migrated note parsing and mapping behavior into `tuner-core` with parity coverage.
- Migrated DSP logic into `tuner-dsp` with explicit `native-audio` and `web-audio` feature gating.
- Added `tuner-web-bridge` WASM exports and API-contract coverage, including config validation hardening.
- Added CI workspace and package validation so library migration artifacts stay stable.
