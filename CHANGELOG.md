# Changelog

All notable changes to the MT2 Draft Assistant project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of MT2 Draft Assistant
- Real-time draft assistance overlay for Monster Train 2
- Deck tracker with composition analysis
- Intelligent card scoring algorithm (0-120 scale)
- Champion and covenant selection
- Card browser with search and filters
- OCR support for automatic card detection (optional)
- Cross-platform support (Windows, macOS, Linux)

### Features

#### Deck Tracker
- Real-time deck composition tracking
- Champion and path selection (11 champions, 2 paths each)
- Ring number and covenant level tracking
- Missing component warnings (frontline, backline clear, scaling)
- Active synergy detection (Shift Loop, Consume Chain, Reform Engine, etc.)
- Unit and spell categorization

#### Scoring Algorithm
- Base value from community tier lists
- Synergy multiplier (1.0x - 1.5x capped)
- Context bonuses based on deck needs
- Champion-specific overrides
- Ring-appropriate adjustments
- Tier rankings (S: 90+, A: 80-89, B: 70-79, C: <70)

#### Card Browser
- Search by name (case-insensitive, partial match)
- Filter by clan (9 clans)
- Filter by rarity (Champion, Rare, Uncommon, Common)
- Base value display with color coding
- Keyword tags

#### Draft Overlay
- Real-time card recommendations
- Score display with tier badges
- Selection reasons breakdown
- Manual input mode
- Toggle visibility with hotkey

#### OCR (Optional)
- Screen capture for card detection
- Image preprocessing (grayscale, threshold, denoise)
- Tesseract OCR integration
- Fuzzy card name matching
- Confidence scoring
- Calibration system

### Technical

#### Backend (Rust)
- Tauri v2 framework
- SQLite database with migrations
- Modular architecture (commands, database, scoring, OCR)
- 53 unit tests covering core functionality
- Optional OCR feature flag

#### Frontend (React/TypeScript)
- React 18 with hooks
- TypeScript for type safety
- Tailwind CSS for styling
- Zustand for state management
- React Query for server state
- 16 unit tests for stores and API

#### Database
- Cards table with all MT2 cards
- Synergies table for card interactions
- Context modifiers for draft-time adjustments
- Champion overrides for path-specific values
- Deck history for ML training (future)

### Supported Clans
- Banished
- Pyreborne
- Luna Coven
- Underlegion
- Lazarus League
- Melting Remnant
- Hellhorned
- Railforged (Destiny of the Railforged expansion)
- Wurmkin

### Champions
- Fel (Banished)
- Talos (Banished)
- Lord Fenix (Pyreborne)
- Lady Gilda (Pyreborne)
- Ekka (Luna Coven)
- Bolete (Underlegion)
- Madame Lionsmane (Underlegion)
- Orechi (Lazarus League)
- Rector Flicker (Melting Remnant)
- Herzal (Railforged)
- Heph (Railforged)

## [0.1.0] - 2026-02-14

### Added
- Initial project setup
- Basic Tauri application structure
- Database schema design
- Scoring algorithm foundation

[Unreleased]: https://github.com/yourusername/mt2-draft-assistant/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/mt2-draft-assistant/releases/tag/v0.1.0
