# MT2 Draft Assistant - AGENT.md

## Project Overview

Monster Train 2 Draft Assistant is a real-time overlay application that provides drafting recommendations during gameplay. Built with Tauri v2, featuring a Rust backend and React frontend.

**Tech Stack:**
- **Backend:** Rust (Tauri v2, SQLite via rusqlite)
- **Frontend:** React + TypeScript + Tailwind CSS
- **OCR:** Tesseract (screen capture + text recognition)
- **Database:** SQLite with migrations

## Development Workflow (TDD)

### The Golden Rule
**Write tests BEFORE implementation. No exceptions.**

### TDD Cycle

```
1. Write a failing test
2. Run test suite (confirm it fails)
3. Write minimal code to make test pass
4. Run test suite (confirm it passes)
5. Refactor while keeping tests green
6. Repeat
```

### Test Categories

#### Unit Tests (Backend - Rust)
Location: `src-tauri/src/*_test.rs`
Run: `cargo test`

```rust
// Example: src-tauri/src/scoring_test.rs
#[test]
fn deadly_plunge_base_value_is_92() {
    let calculator = ScoreCalculator::new();
    let score = calculator.calculate_base("deadly_plunge");
    assert_eq!(score, 92);
}
```

#### Unit Tests (Frontend - TypeScript)
Location: `src/**/*.test.ts`
Run: `npm test`

```typescript
// Example: src/lib/scoring.test.ts
import { calculateScore } from './scoring';

test('Deadly Plunge has base value 92', () => {
  const result = calculateScore('deadly_plunge', [], 'Fel', 1, 10);
  expect(result.score).toBe(92);
});
```

#### Integration Tests
Location: `tests/integration/`
Run: `cargo test --test integration`

```rust
#[test]
fn full_draft_scenario_with_synergies() {
    // Test complete draft flow
}
```

#### E2E Tests
Location: `e2e/`
Run: `npm run test:e2e`

## Code Organization (DRY Principles)

### Architecture Overview

```
mt2-overlay/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs               # Main library entry
│   │   ├── main.rs              # Binary entry
│   │   ├── database/            # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs        # Schema definitions
│   │   │   ├── migrations/      # DB migrations
│   │   │   └── repository.rs    # Data access
│   │   ├── scoring/             # Scoring algorithm
│   │   │   ├── mod.rs
│   │   │   ├── calculator.rs    # Main calculator
│   │   │   ├── synergies.rs     # Synergy detection
│   │   │   └── context.rs       # Context modifiers
│   │   ├── ocr/                 # OCR pipeline
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs       # Screen capture
│   │   │   ├── preprocess.rs    # Image preprocessing
│   │   │   └── recognize.rs     # Text recognition
│   │   └── commands/            # Tauri commands
│   │       ├── mod.rs
│   │       ├── cards.rs         # Card queries
│   │       ├── scoring.rs       # Scoring commands
│   │       └── ocr.rs           # OCR commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                          # React frontend
│   ├── components/
│   │   ├── deck-tracker/        # Deck tracker panel
│   │   ├── draft-overlay/       # Draft recommendation overlay
│   │   ├── card-display/        # Card display components
│   │   └── shared/              # Shared UI components
│   ├── hooks/
│   │   ├── useCards.ts          # Card data hooks
│   │   ├── useScoring.ts        # Scoring hooks
│   │   └── useOCR.ts            # OCR hooks
│   ├── lib/
│   │   ├── database.ts          # Database client
│   │   ├── scoring.ts           # Scoring utilities
│   │   └── utils.ts             # General utilities
│   ├── types/
│   │   └── index.ts             # TypeScript types
│   ├── App.tsx
│   └── main.tsx
├── tests/                        # Test files
├── docs/                         # Documentation
└── AGENT.md                      # This file
```

### DRY Principles

1. **Single Source of Truth:**
   - Card data exists ONLY in database
   - Scoring logic exists ONLY in backend
   - Types are shared via TypeScript definitions

2. **No Magic Numbers:**
   ```rust
   // BAD
   if score > 90 { /* ... */ }
   
   // GOOD
   const S_TIER_THRESHOLD: i32 = 90;
   if score > S_TIER_THRESHOLD { /* ... */ }
   ```

3. **Shared Constants:**
   ```rust
   // src-tauri/src/scoring/constants.rs
   pub const SYNERGY_CAP: f64 = 1.5;
   pub const MISSING_FRONTLINE_BONUS: i32 = 15;
   pub const MAX_SCORE: i32 = 120;
   ```

4. **Database Migrations:**
   - All schema changes via migrations
   - Never modify schema directly
   - Version control all migrations

## Naming Conventions

### Rust
- **Structs:** PascalCase (`ScoreCalculator`, `CardRepository`)
- **Functions:** snake_case (`calculate_score`, `get_card_by_id`)
- **Constants:** SCREAMING_SNAKE_CASE (`S_TIER_THRESHOLD`)
- **Modules:** snake_case (`scoring/calculator`)
- **Tests:** `test_` prefix or `#[test]` attribute

### TypeScript/React
- **Components:** PascalCase (`DeckTracker`, `DraftOverlay`)
- **Hooks:** camelCase starting with `use` (`useCards`, `useScoring`)
- **Functions:** camelCase (`calculateScore`, `getCardById`)
- **Types/Interfaces:** PascalCase (`Card`, `ScoringResult`)
- **Constants:** SCREAMING_SNAKE_CASE (`S_TIER_THRESHOLD`)

## Documentation Standards

### Code Comments

```rust
/// Calculates the final draft score for a card.
/// 
/// # Arguments
/// * `card_id` - The unique identifier of the card
/// * `current_deck` - List of card IDs already in the deck
/// * `champion` - Current champion and path
/// * `ring_number` - Current ring (1-9)
/// * `covenant` - Covenant level (0-25)
/// 
/// # Returns
/// The final score from 0-120
/// 
/// # Example
/// ```
/// let score = calculator.calculate("deadly_plunge", &[], "Fel", 1, 10);
/// assert_eq!(score, 92);
/// ```
pub fn calculate(&self, card_id: &str, ...) -> i32 {
    // Implementation
}
```

### README Files

Each module should have a `README.md`:
```markdown
# Module Name

## Purpose
Brief description of what this module does.

## Key Components
- Component A: Does X
- Component B: Does Y

## Usage Example
```rust
// Example code
```

## Testing
```bash
# How to run tests for this module
```
```

## Git Workflow

### Branch Naming
- `feature/scoring-algorithm`
- `bugfix/ocr-latency`
- `docs/api-reference`
- `test/integration-tests`

### Commit Messages
```
type(scope): subject

body (optional)

footer (optional)
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `test`: Adding tests
- `docs`: Documentation
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Build/config changes

Examples:
```
feat(scoring): implement synergy multiplier calculation

test(ocr): add screen capture integration tests

docs(readme): add installation instructions
```

### Pull Request Template

```markdown
## Summary
Brief description of changes

## Type of Change
- [ ] Feature
- [ ] Bugfix
- [ ] Refactor
- [ ] Test
- [ ] Documentation

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] E2E tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows DRY principles
- [ ] Tests written before implementation
- [ ] Documentation updated
- [ ] No console.logs or debug code
- [ ] Types are properly defined
```

## Testing Checklist

Before marking a task complete:

- [ ] Unit tests written and passing
- [ ] Integration tests (if applicable)
- [ ] Edge cases handled
- [ ] Error handling tested
- [ ] Manual testing completed
- [ ] Documentation updated

## Common Commands

```bash
# Setup
cd mt2-overlay
npm install
cargo build

# Development
npm run tauri dev          # Start dev server
npm run tauri build        # Build for production

# Testing
npm test                   # Frontend tests
cargo test                 # Backend tests
cargo test -- --nocapture  # Tests with output

# Database
cargo run --bin migrate    # Run migrations
cargo run --bin seed       # Seed database

# Code Quality
cargo clippy               # Rust linting
cargo fmt                  # Rust formatting
npm run lint               # ESLint
npm run format             # Prettier
```

## Resources

- [Tauri v2 Docs](https://v2.tauri.app/)
- [Monster Train 2 Wiki](https://monstertrain2.miraheze.org/)
- [SQLite Schema Design](https://sqlite.org/lang.html)
- [Tesseract OCR](https://github.com/tesseract-ocr/tesseract)
