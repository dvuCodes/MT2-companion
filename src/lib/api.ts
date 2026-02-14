/**
 * MT2 Draft Assistant - Tauri IPC API Layer
 * 
 * This module provides type-safe wrappers around all Tauri invoke commands.
 * All backend communication should go through this API layer.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  Card,
  DraftScoreRequest,
  DraftScoreResponse,
  CardDetectionResult,
  CalibrationResult,
  OverlayPosition,
} from '../types';

// ============================================================================
// Error Handling
// ============================================================================

class ApiError extends Error {
  constructor(
    message: string,
    public readonly command: string,
    public readonly originalError: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    console.error(`API Error [${command}]:`, error);
    throw new ApiError(
      `Failed to execute ${command}: ${error instanceof Error ? error.message : String(error)}`,
      command,
      error
    );
  }
}

// ============================================================================
// Card API
// ============================================================================

/**
 * Get a single card by its exact name
 */
export async function getCardByName(name: string): Promise<Card | null> {
  const response = await invokeCommand<Card | null>('get_card_by_name', { name });
  return response;
}

/**
 * Get all cards belonging to a specific clan
 */
export async function getCardsByClan(clan: string): Promise<Card[]> {
  const response = await invokeCommand<Card[]>('get_cards_by_clan', { clan });
  return response;
}

/**
 * Search cards by query string (matches name, keywords, description)
 */
export async function searchCards(query: string): Promise<Card[]> {
  const response = await invokeCommand<Card[]>('search_cards', { query });
  return response;
}

/**
 * Get all cards in the database
 */
export async function getAllCards(): Promise<Card[]> {
  const response = await invokeCommand<Card[]>('get_all_cards');
  return response;
}

// ============================================================================
// Scoring API
// ============================================================================

/**
 * Calculate draft score for a card in the current context
 */
export async function calculateDraftScore(
  request: DraftScoreRequest
): Promise<DraftScoreResponse> {
  // Convert camelCase to snake_case for Rust compatibility
  const rustRequest = {
    card_id: request.cardId,
    current_deck: request.currentDeck,
    champion: request.champion,
    ring_number: request.ringNumber,
    covenant: request.covenant,
  };
  
  const response = await invokeCommand<DraftScoreResponse>('calculate_draft_score', {
    request: rustRequest,
  });
  return response;
}

/**
 * Get synergies for a specific card
 */
export async function getSynergies(cardId: string): Promise<string[]> {
  const response = await invokeCommand<string[]>('get_synergies', { cardId });
  return response;
}

/**
 * Get available context modifiers for scoring
 */
export async function getContextModifiers(): Promise<string[]> {
  const response = await invokeCommand<string[]>('get_context_modifiers');
  return response;
}

// ============================================================================
// OCR API
// ============================================================================

/**
 * Detect cards currently visible on screen using OCR
 */
export async function detectCardsOnScreen(): Promise<CardDetectionResult> {
  const response = await invokeCommand<CardDetectionResult>('detect_cards_on_screen');
  return response;
}

/**
 * Calibrate OCR detection regions
 */
export async function calibrateOcrRegions(): Promise<CalibrationResult> {
  const response = await invokeCommand<CalibrationResult>('calibrate_ocr_regions');
  return response;
}

// ============================================================================
// Window/Overlay API
// ============================================================================

/**
 * Toggle overlay window visibility
 * @returns New visibility state (true = visible)
 */
export async function toggleOverlay(): Promise<boolean> {
  const response = await invokeCommand<boolean>('toggle_overlay');
  return response;
}

/**
 * Show the overlay window
 */
export async function showOverlay(): Promise<void> {
  await invokeCommand<void>('show_overlay');
}

/**
 * Hide the overlay window
 */
export async function hideOverlay(): Promise<void> {
  await invokeCommand<void>('hide_overlay');
}

/**
 * Set overlay window position
 */
export async function setOverlayPosition(position: OverlayPosition): Promise<void> {
  await invokeCommand<void>('set_overlay_position', { position });
}

// ============================================================================
// Batch Operations
// ============================================================================

/**
 * Fetch cards by their IDs in a single batch operation
 */
export async function getCardsByIds(cardIds: string[]): Promise<Card[]> {
  const cards: Card[] = [];
  
  // Fetch cards in parallel for better performance
  const results = await Promise.allSettled(
    cardIds.map(id => getCardByName(id))
  );
  
  results.forEach((result) => {
    if (result.status === 'fulfilled' && result.value) {
      cards.push(result.value);
    }
  });
  
  return cards;
}

/**
 * Calculate scores for multiple cards at once
 */
export async function calculateDraftScores(
  cardIds: string[],
  request: Omit<DraftScoreRequest, 'cardId'>
): Promise<Map<string, DraftScoreResponse>> {
  const scores = new Map<string, DraftScoreResponse>();
  
  const results = await Promise.allSettled(
    cardIds.map(async (cardId) => {
      const score = await calculateDraftScore({ ...request, cardId });
      return { cardId, score };
    })
  );
  
  results.forEach((result) => {
    if (result.status === 'fulfilled') {
      scores.set(result.value.cardId, result.value.score);
    }
  });
  
  return scores;
}

// ============================================================================
// Export/Import API
// ============================================================================

export interface DeckExportData {
  version: string;
  exportedAt: string;
  champion: string;
  championPath: string;
  covenant: number;
  ring: number;
  cards: ExportedCard[];
  metadata: ExportMetadata;
}

export interface ExportedCard {
  id: string;
  name: string;
  draftOrder: number;
  ringNumber: number;
}

export interface ExportMetadata {
  totalValue: number;
  unitCount: number;
  spellCount: number;
  synergyCount: number;
}

export interface ExportFormat {
  id: string;
  name: string;
  extension: string;
  description: string;
}

/**
 * Export deck to JSON file
 */
export async function exportDeck(deckData: DeckExportData, filePath: string): Promise<void> {
  await invokeCommand('export_deck', { deckData, filePath });
}

/**
 * Import deck from JSON file
 */
export async function importDeck(filePath: string): Promise<DeckExportData> {
  return await invokeCommand<DeckExportData>('import_deck', { filePath });
}

/**
 * Export deck history to CSV
 */
export async function exportHistoryCsv(filePath: string): Promise<void> {
  await invokeCommand('export_history_csv', { filePath });
}

/**
 * Get available export formats
 */
export async function getExportFormats(): Promise<ExportFormat[]> {
  return await invokeCommand<ExportFormat[]>('get_export_formats');
}
