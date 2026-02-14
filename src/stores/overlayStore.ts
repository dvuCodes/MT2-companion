/**
 * Overlay Store - Manages the draft overlay state
 * 
 * Handles:
 * - Overlay visibility
 * - Currently detected cards
 * - Scoring results for detected cards
 * - OCR state
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type { Card, ScoredCard, DraftScoreResponse, OverlayPosition } from '../types';
import * as api from '../lib/api';

// ============================================================================
// Types
// ============================================================================

interface DetectedCard {
  name: string;
  confidence: number;
  cardData?: Card;
}

interface OverlayState {
  // Visibility
  isVisible: boolean;
  
  // Detected cards from OCR
  detectedCards: DetectedCard[];
  isDetecting: boolean;
  lastDetectionTime: number | null;
  detectionError: string | null;
  
  // Scoring results
  scoredCards: Map<string, DraftScoreResponse>;
  isScoring: boolean;
  scoringError: string | null;
  
  // Position
  position: OverlayPosition;
  
  // OCR calibration
  isCalibrating: boolean;
  calibrationError: string | null;
}

interface OverlayActions {
  // Visibility
  show: () => Promise<void>;
  hide: () => Promise<void>;
  toggle: () => Promise<boolean>;
  setVisible: (visible: boolean) => void;
  
  // Detection
  detectCards: () => Promise<void>;
  setDetectedCards: (cards: DetectedCard[]) => void;
  clearDetectedCards: () => void;
  
  // Scoring
  calculateScores: (champion: string, ring: number, covenant: number, currentDeck: string[]) => Promise<void>;
  clearScores: () => void;
  
  // Position
  setPosition: (position: OverlayPosition) => Promise<void>;
  resetPosition: () => void;
  
  // Calibration
  calibrate: () => Promise<void>;
  
  // Errors
  clearErrors: () => void;
}

interface OverlayStore extends OverlayState, OverlayActions {
  // Computed
  getScoredCards: () => ScoredCard[];
  hasDetectedCards: () => boolean;
  hasScores: () => boolean;
}

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_POSITION: OverlayPosition = { x: 0, y: 0 };

// Minimum time between detections (ms)
const DETECTION_COOLDOWN = 1000;

// ============================================================================
// Helper Functions
// ============================================================================

function getTierColor(tier: string): string {
  switch (tier.toUpperCase()) {
    case 'S': return 'text-yellow-400 border-yellow-400';
    case 'A': return 'text-gray-300 border-gray-300';
    case 'B': return 'text-orange-400 border-orange-400';
    case 'C': return 'text-gray-500 border-gray-500';
    case 'D': return 'text-red-400 border-red-400';
    default: return 'text-gray-500 border-gray-500';
  }
}

function getScoreColor(score: number): string {
  if (score >= 90) return 'text-yellow-400';
  if (score >= 80) return 'text-gray-200';
  if (score >= 70) return 'text-orange-400';
  return 'text-gray-500';
}

// ============================================================================
// Store
// ============================================================================

export const useOverlayStore = create<OverlayStore>()(
  devtools(
    (set, get) => ({
      // Initial state
      isVisible: false,
      detectedCards: [],
      isDetecting: false,
      lastDetectionTime: null,
      detectionError: null,
      scoredCards: new Map(),
      isScoring: false,
      scoringError: null,
      position: { ...DEFAULT_POSITION },
      isCalibrating: false,
      calibrationError: null,

      // Visibility
      show: async () => {
        try {
          await api.showOverlay();
          set({ isVisible: true });
        } catch (err) {
          console.error('Failed to show overlay:', err);
          set({ 
            detectionError: err instanceof Error ? err.message : 'Failed to show overlay'
          });
        }
      },

      hide: async () => {
        try {
          await api.hideOverlay();
          set({ isVisible: false });
        } catch (err) {
          console.error('Failed to hide overlay:', err);
        }
      },

      toggle: async () => {
        try {
          const newState = await api.toggleOverlay();
          set({ isVisible: newState });
          return newState;
        } catch (err) {
          console.error('Failed to toggle overlay:', err);
          set({ 
            detectionError: err instanceof Error ? err.message : 'Failed to toggle overlay'
          });
          return get().isVisible;
        }
      },

      setVisible: (isVisible) => set({ isVisible }),

      // Detection
      detectCards: async () => {
        const now = Date.now();
        const lastDetection = get().lastDetectionTime;
        
        // Enforce cooldown
        if (lastDetection && now - lastDetection < DETECTION_COOLDOWN) {
          return;
        }
        
        set({ 
          isDetecting: true, 
          detectionError: null,
        });
        
        try {
          const result = await api.detectCardsOnScreen();
          
          // Convert detection results to detected cards
          const detectedCards: DetectedCard[] = result.detectedCards.map(name => ({
            name,
            confidence: result.confidence,
          }));
          
          // Try to fetch card data for detected cards
          const cardPromises = detectedCards.map(async (detected) => {
            try {
              const cardData = await api.getCardByName(detected.name);
              return { ...detected, cardData: cardData || undefined };
            } catch {
              return detected;
            }
          });
          
          const cardsWithData = await Promise.all(cardPromises);
          
          set({
            detectedCards: cardsWithData,
            isDetecting: false,
            lastDetectionTime: Date.now(),
          });
        } catch (err) {
          set({ 
            isDetecting: false,
            detectionError: err instanceof Error ? err.message : 'Failed to detect cards',
          });
        }
      },

      setDetectedCards: (detectedCards) => {
        set({ detectedCards });
      },

      clearDetectedCards: () => {
        set({ 
          detectedCards: [],
          scoredCards: new Map(),
        });
      },

      // Scoring
      calculateScores: async (champion, ring, covenant, currentDeck) => {
        const { detectedCards } = get();
        
        if (detectedCards.length === 0) return;
        
        set({ isScoring: true, scoringError: null });
        
        try {
          // Get card IDs for detected cards (use name as fallback ID)
          const cardIds = detectedCards
            .map(c => c.cardData?.id || c.name)
            .filter(Boolean);
          
          const scores = await api.calculateDraftScores(cardIds, {
            currentDeck,
            champion,
            ringNumber: ring,
            covenant,
          });
          
          set({ 
            scoredCards: scores,
            isScoring: false,
          });
        } catch (err) {
          set({ 
            isScoring: false,
            scoringError: err instanceof Error ? err.message : 'Failed to calculate scores',
          });
        }
      },

      clearScores: () => {
        set({ scoredCards: new Map() });
      },

      // Position
      setPosition: async (position) => {
        try {
          await api.setOverlayPosition(position);
          set({ position });
        } catch (err) {
          console.error('Failed to set overlay position:', err);
        }
      },

      resetPosition: () => {
        set({ position: { ...DEFAULT_POSITION } });
      },

      // Calibration
      calibrate: async () => {
        set({ isCalibrating: true, calibrationError: null });
        
        try {
          const result = await api.calibrateOcrRegions();
          
          if (!result.success) {
            set({ 
              isCalibrating: false,
              calibrationError: result.message,
            });
          } else {
            set({ isCalibrating: false });
          }
        } catch (err) {
          set({ 
            isCalibrating: false,
            calibrationError: err instanceof Error ? err.message : 'Calibration failed',
          });
        }
      },

      // Errors
      clearErrors: () => {
        set({ 
          detectionError: null,
          scoringError: null,
          calibrationError: null,
        });
      },

      // Computed
      getScoredCards: () => {
        const { detectedCards, scoredCards } = get();
        
        return detectedCards.map(detected => {
          const cardId = detected.cardData?.id || detected.name;
          const score = scoredCards.get(cardId);
          
          return {
            id: cardId,
            name: detected.name,
            clan: detected.cardData?.clan || 'Unknown',
            cardType: detected.cardData?.cardType || 'Unknown',
            rarity: detected.cardData?.rarity || 'Common',
            cost: detected.cardData?.cost || null,
            baseValue: detected.cardData?.baseValue || 0,
            tempoScore: detected.cardData?.tempoScore || 0,
            valueScore: detected.cardData?.valueScore || 0,
            keywords: detected.cardData?.keywords || [],
            description: detected.cardData?.description || '',
            expansion: detected.cardData?.expansion || '',
            score: score?.score || 0,
            tier: score?.tier || '?',
            reasons: score?.reasons || ['No score available'],
          } as ScoredCard;
        });
      },

      hasDetectedCards: () => {
        return get().detectedCards.length > 0;
      },

      hasScores: () => {
        return get().scoredCards.size > 0;
      },
    }),
    { name: 'mt2-overlay-store' }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectOverlayVisibility = (state: OverlayStore) => ({
  isVisible: state.isVisible,
});

export const selectDetectedCards = (state: OverlayStore) => state.detectedCards;

export const selectDetectionStatus = (state: OverlayStore) => ({
  isDetecting: state.isDetecting,
  detectionError: state.detectionError,
});

export const selectScoringStatus = (state: OverlayStore) => ({
  isScoring: state.isScoring,
  scoringError: state.scoringError,
  scoredCards: state.scoredCards,
});

// Export helpers
export { getTierColor, getScoreColor };
