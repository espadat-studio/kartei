# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.2](https://github.com/espadat-studio/kartei/compare/v2.0.1...v2.0.2) - 2026-10-26

### Other

- *(deps)* update hk to v2.8.1 ([#98](https://github.com/espadat-studio/kartei/pull/98))
- *(deps)* update all dependencies ([#97](https://github.com/espadat-studio/kartei/pull/97))
- *(renovate)* batch all updates into one weekly pr ([#96](https://github.com/espadat-studio/kartei/pull/96))
- *(deps)* update release-plz/action digest to 6caa8d3 ([#94](https://github.com/espadat-studio/kartei/pull/94))
- *(deps)* update dependency astro to v7.4.1 ([#95](https://github.com/espadat-studio/kartei/pull/95))
- *(deps)* update jdx/mise-action digest to fc42bcc ([#92](https://github.com/espadat-studio/kartei/pull/92))
- *(deps)* update hk to v2.7.1 ([#93](https://github.com/espadat-studio/kartei/pull/93))
- *(deps)* update hk to v2.7.0 ([#89](https://github.com/espadat-studio/kartei/pull/89))
- *(deps)* update dependency aqua:dprint/dprint to v0.64.0 ([#91](https://github.com/espadat-studio/kartei/pull/91))
- *(deps)* update taiki-e/install-action digest to 8cca091 ([#90](https://github.com/espadat-studio/kartei/pull/90))
- *(deps)* update jdx/mise-action digest to 9b200bc ([#88](https://github.com/espadat-studio/kartei/pull/88))
- *(deps)* update all non-major dependencies ([#86](https://github.com/espadat-studio/kartei/pull/86))
- *(deps)* update dtolnay/rust-toolchain digest to b725eda ([#87](https://github.com/espadat-studio/kartei/pull/87))
- *(deps)* update node.js to v24.22.0 ([#85](https://github.com/espadat-studio/kartei/pull/85))
- *(deps)* update hk to v2.6.0 ([#82](https://github.com/espadat-studio/kartei/pull/82))
- *(deps)* update jdx/mise-action digest to 08ae2a0 ([#81](https://github.com/espadat-studio/kartei/pull/81))
- *(deps)* update actions/create-github-app-token digest to 64ab729 ([#83](https://github.com/espadat-studio/kartei/pull/83))
- *(deps)* update release-plz/action digest to 8921286 ([#84](https://github.com/espadat-studio/kartei/pull/84))
- *(deps)* update jdx/mise-action action to v5 ([#80](https://github.com/espadat-studio/kartei/pull/80))
- *(deps)* update hk to v2.5.0 ([#79](https://github.com/espadat-studio/kartei/pull/79))

## [2.0.1](https://github.com/espadat-studio/kartei/compare/v2.0.0...v2.0.1) - 2026-10-12

### Other

- use org github app client id ([#77](https://github.com/espadat-studio/kartei/pull/77))
- *(deps)* update all non-major dependencies ([#76](https://github.com/espadat-studio/kartei/pull/76))
- *(deps)* bump source-map-js from 1.2.1 to 1.2.2 in /docs ([#74](https://github.com/espadat-studio/kartei/pull/74))
- *(deps)* update taiki-e/install-action digest to 1168f68 ([#73](https://github.com/espadat-studio/kartei/pull/73))
- *(deps)* bump sharp from 0.35.4 to 0.35.5 in /docs ([#75](https://github.com/espadat-studio/kartei/pull/75))
- *(deps)* update taiki-e/install-action digest to ab68953 ([#72](https://github.com/espadat-studio/kartei/pull/72))
- *(deps)* update release-plz/action digest to f393a7b ([#71](https://github.com/espadat-studio/kartei/pull/71))
- *(deps)* update dtolnay/rust-toolchain digest to 686976e ([#69](https://github.com/espadat-studio/kartei/pull/69))
- *(deps)* update actions/setup-node digest to 949feb2 ([#68](https://github.com/espadat-studio/kartei/pull/68))
- research how baikal, syncers and pimdir fit together ([#67](https://github.com/espadat-studio/kartei/pull/67))
- lead the copy with lossless edits ([#66](https://github.com/espadat-studio/kartei/pull/66))

## [2.0.0](https://github.com/espadat-studio/kartei/compare/v1.0.0...v2.0.0) - 2026-09-26

### Added

- defer reloads during edits and follow removed cards ([#64](https://github.com/espadat-studio/kartei/pull/64))
- [**breaking**] complete addresses with kartei query ([#63](https://github.com/espadat-studio/kartei/pull/63))
- reload the address book when files change ([#62](https://github.com/espadat-studio/kartei/pull/62))
- rank fuzzy search results ([#61](https://github.com/espadat-studio/kartei/pull/61))
- parse the command line with clap ([#60](https://github.com/espadat-studio/kartei/pull/60))
- show an empty state and add --version ([#51](https://github.com/espadat-studio/kartei/pull/51))

### Other

- record edit-interrupt prompt as out of scope ([#65](https://github.com/espadat-studio/kartei/pull/65))
- sort cargo dependencies
- *(docs)* bump the docs theme to v0.4.0
- drop pre-release notices for 1.0.0
- release v1.0.0 ([#38](https://github.com/espadat-studio/kartei/pull/38))

### Changed

- `query` is a reserved word: `kartei query` completes addresses, so open a directory named `query` as `./query`

## [1.0.0](https://github.com/espadat-studio/kartei/compare/v0.0.1...v1.0.0) - 2026-09-25

### Added

- raw edit a card in $EDITOR ([#49](https://github.com/espadat-studio/kartei/pull/49))
- delete a card ([#48](https://github.com/espadat-studio/kartei/pull/48))
- edit urls ([#47](https://github.com/espadat-studio/kartei/pull/47))
- open a single vcf file as the address book ([#40](https://github.com/espadat-studio/kartei/pull/40))
- skip a defective card inside a bundle ([#39](https://github.com/espadat-studio/kartei/pull/39))
- browse and edit cards inside bundles ([#37](https://github.com/espadat-studio/kartei/pull/37))

### Other

- bump version to 1.0.0 ([#50](https://github.com/espadat-studio/kartei/pull/50))
- document bundles and the thunderbird round trip ([#41](https://github.com/espadat-studio/kartei/pull/41))
- define bundles and record in-place bundle edits

## [0.0.1](https://github.com/espadat-studio/kartei/compare/v0.0.0...v0.0.1) - 2026-09-25

### Added

- complete the edit form, new cards, conflicts and copy ([#31](https://github.com/espadat-studio/kartei/pull/31))
- edit and save single-value fields ([#30](https://github.com/espadat-studio/kartei/pull/30))
- report skipped cards ([#27](https://github.com/espadat-studio/kartei/pull/27))
- show full card details with apple quirks ([#26](https://github.com/espadat-studio/kartei/pull/26))
- browse an address book read-only ([#25](https://github.com/espadat-studio/kartei/pull/25))

### Other

- document keymap and usage ([#32](https://github.com/espadat-studio/kartei/pull/32))
- add starlight docs site ([#24](https://github.com/espadat-studio/kartei/pull/24))
- add renovate config ([#22](https://github.com/espadat-studio/kartei/pull/22))
- add check, test and pr title workflows ([#21](https://github.com/espadat-studio/kartei/pull/21))
- scaffold cargo crate and local tooling ([#20](https://github.com/espadat-studio/kartei/pull/20))
- add mvp plan, adr and glossary ([#7](https://github.com/espadat-studio/kartei/pull/7))
- *(research)* add contacts tui and build research notes ([#1](https://github.com/espadat-studio/kartei/pull/1))
- Initial commit
