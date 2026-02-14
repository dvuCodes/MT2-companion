/**
 * MT2 Draft Assistant - Type Definitions
 * 
 * These types mirror the Rust backend types for type-safe IPC communication.
 */

// ============================================================================
// Card Types
// ============================================================================

export interface Card {
  id: string;
  name: string;
  clan: string;
  cardType: string;
  rarity: string;
  cost: number | null;
  baseValue: number;
  tempoScore: number;
  valueScore: number;
  keywords: string[];
  description: string;
  expansion: string;
}

export interface DeckCard extends Card {
  draftOrder: number;
  ringNumber: number;
}

// ============================================================================
// Scoring Types
// ============================================================================

export interface DraftScore {
  score: number;
  tier: string;
  reasons: string[];
}

export interface DraftScoreRequest {
  cardId: string;
  currentDeck: string[];
  champion: string;
  ringNumber: number;
  covenant: number;
}

export interface DraftScoreResponse {
  score: number;
  tier: string;
  reasons: string[];
}

export interface ScoredCard extends Card {
  score: number;
  tier: string;
  reasons: string[];
}

// ============================================================================
// OCR Types
// ============================================================================

export interface CardDetectionResult {
  detectedCards: string[];
  confidence: number;
}

export interface CalibrationResult {
  success: boolean;
  message: string;
}

// ============================================================================
// Window/Overlay Types
// ============================================================================

export interface OverlayPosition {
  x: number;
  y: number;
}

export type OverlayMode = 'automatic' | 'manual' | 'hybrid';

// ============================================================================
// Settings Types
// ============================================================================

export interface AppSettings {
  ocrMode: OverlayMode;
  covenantLevel: number;
  overlayOpacity: number;
  showCardArt: boolean;
  enableSoundAlerts: boolean;
  theme: 'dark' | 'light' | 'system';
}

export const DEFAULT_SETTINGS: AppSettings = {
  ocrMode: 'automatic',
  covenantLevel: 10,
  overlayOpacity: 0.95,
  showCardArt: true,
  enableSoundAlerts: false,
  theme: 'dark',
};

// ============================================================================
// Champion Types
// ============================================================================

export interface Champion {
  id: string;
  name: string;
  clan: string;
  paths: string[];
}

export const CHAMPIONS: Champion[] = [
  { id: 'fel', name: 'Fel', clan: 'Hellhorned', paths: ['Unchained', 'Savior'] },
  { id: 'talos', name: 'Talos', clan: 'Railforged', paths: ['Unchained', 'Savior'] },
  { id: 'lord_fenix', name: 'Lord Fenix', clan: 'Pyreborne', paths: ['Unchained', 'Savior'] },
  { id: 'lady_gilda', name: 'Lady Gilda', clan: 'Lazarus League', paths: ['Unchained', 'Savior'] },
  { id: 'ekka', name: 'Ekka', clan: 'Wurmkin', paths: ['Unchained', 'Savior'] },
  { id: 'bolete', name: 'Bolete', clan: 'Melting Remnant', paths: ['Unchained', 'Savior'] },
  { id: 'madame_lionsmane', name: 'Madame Lionsmane', clan: 'Luna Coven', paths: ['Unchained', 'Savior'] },
  { id: 'orechi', name: 'Orechi', clan: 'Underlegion', paths: ['Unchained', 'Savior'] },
  { id: 'rector_flicker', name: 'Rector Flicker', clan: 'Melting Remnant', paths: ['Unchained', 'Savior'] },
  { id: 'herszal', name: 'Herszal', clan: 'Wurmkin', paths: ['Unchained', 'Savior'] },
  { id: 'heph', name: 'Heph', clan: 'Hellhorned', paths: ['Unchained', 'Savior'] },
];

// ============================================================================
// Deck Analysis Types
// ============================================================================

export interface SynergyCheck {
  name: string;
  active: boolean;
  description?: string;
}

export interface DeckAnalysis {
  hasFrontline: boolean;
  hasBacklineClear: boolean;
  hasScaling: boolean;
  activeSynergies: SynergyCheck[];
  totalCards: number;
  unitCount: number;
  spellCount: number;
  averageValue: number;
}

// ============================================================================
// API Response Types
// ============================================================================

export interface ApiState<T> {
  data: T | null;
  isLoading: boolean;
  error: string | null;
}

export interface AsyncState extends ApiState<unknown> {
  lastUpdated?: number;
}
