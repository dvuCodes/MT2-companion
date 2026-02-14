/**
 * Card Store - Manages card data from the backend
 * 
 * Handles:
 * - Loading all cards from the database
 * - Searching and filtering cards
 * - Caching card data
 * - Loading states and errors
 */

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type { Card, DraftScoreResponse } from '../types';
import * as api from '../lib/api';

// ============================================================================
// Types
// ============================================================================

interface CardCache {
  [key: string]: Card;
}

interface CardFilters {
  searchQuery: string;
  selectedClan: string;
  selectedRarity: string;
  selectedType: string;
  minValue: number | null;
  maxValue: number | null;
}

interface CardState {
  // Data
  cards: Card[];
  cache: CardCache;
  scoredCards: Map<string, DraftScoreResponse>;
  
  // Filters
  filters: CardFilters;
  
  // UI state
  isLoading: boolean;
  isSearching: boolean;
  error: string | null;
  lastFetched: number | null;
}

interface CardActions {
  // Data loading
  fetchAllCards: () => Promise<void>;
  fetchCardsByClan: (clan: string) => Promise<void>;
  searchCards: (query: string) => Promise<void>;
  fetchCardByName: (name: string) => Promise<Card | null>;
  
  // Scoring
  calculateScores: (cardIds: string[], champion: string, ring: number, covenant: number) => Promise<void>;
  getCardScore: (cardId: string) => DraftScoreResponse | undefined;
  
  // Cache management
  cacheCards: (cards: Card[]) => void;
  getCachedCard: (id: string) => Card | undefined;
  clearCache: () => void;
  
  // Filter actions
  setSearchQuery: (query: string) => void;
  setSelectedClan: (clan: string) => void;
  setSelectedRarity: (rarity: string) => void;
  setSelectedType: (type: string) => void;
  setValueRange: (min: number | null, max: number | null) => void;
  resetFilters: () => void;
  
  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

interface CardStore extends CardState, CardActions {
  // Computed
  getFilteredCards: () => Card[];
  getClans: () => string[];
  getRarities: () => string[];
  getCardTypes: () => string[];
}

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_FILTERS: CardFilters = {
  searchQuery: '',
  selectedClan: 'All',
  selectedRarity: 'All',
  selectedType: 'All',
  minValue: null,
  maxValue: null,
};

// Cache expiration time (5 minutes)
// const CACHE_EXPIRY = 5 * 60 * 1000; // Reserved for future cache invalidation

// ============================================================================
// Store
// ============================================================================

export const useCardStore = create<CardStore>()(
  devtools(
    (set, get) => ({
      // Initial state
      cards: [],
      cache: {},
      scoredCards: new Map(),
      filters: { ...DEFAULT_FILTERS },
      isLoading: false,
      isSearching: false,
      error: null,
      lastFetched: null,

      // Data loading
      fetchAllCards: async () => {
        set({ isLoading: true, error: null });
        
        try {
          const cards = await api.getAllCards();
          
          // Update cache
          const cacheUpdates: CardCache = {};
          cards.forEach(card => {
            cacheUpdates[card.id] = card;
          });
          
          set({ 
            cards,
            cache: { ...get().cache, ...cacheUpdates },
            isLoading: false,
            lastFetched: Date.now(),
          });
        } catch (err) {
          set({ 
            error: err instanceof Error ? err.message : 'Failed to fetch cards',
            isLoading: false,
          });
        }
      },

      fetchCardsByClan: async (clan) => {
        if (clan === 'All') {
          await get().fetchAllCards();
          return;
        }
        
        set({ isLoading: true, error: null });
        
        try {
          const cards = await api.getCardsByClan(clan);
          
          // Update cache
          const cacheUpdates: CardCache = {};
          cards.forEach(card => {
            cacheUpdates[card.id] = card;
          });
          
          set({ 
            cards,
            cache: { ...get().cache, ...cacheUpdates },
            isLoading: false,
          });
        } catch (err) {
          set({ 
            error: err instanceof Error ? err.message : 'Failed to fetch cards by clan',
            isLoading: false,
          });
        }
      },

      searchCards: async (query) => {
        if (!query.trim()) {
          await get().fetchAllCards();
          return;
        }
        
        set({ isSearching: true, error: null });
        
        try {
          const cards = await api.searchCards(query);
          
          // Update cache
          const cacheUpdates: CardCache = {};
          cards.forEach(card => {
            cacheUpdates[card.id] = card;
          });
          
          set({ 
            cards,
            cache: { ...get().cache, ...cacheUpdates },
            isSearching: false,
          });
        } catch (err) {
          set({ 
            error: err instanceof Error ? err.message : 'Failed to search cards',
            isSearching: false,
          });
        }
      },

      fetchCardByName: async (name) => {
        // Check cache first
        const cached = Object.values(get().cache).find(c => 
          c.name.toLowerCase() === name.toLowerCase()
        );
        if (cached) return cached;
        
        try {
          const card = await api.getCardByName(name);
          if (card) {
            get().cacheCards([card]);
          }
          return card;
        } catch (err) {
          set({ 
            error: err instanceof Error ? err.message : `Failed to fetch card: ${name}`,
          });
          return null;
        }
      },

      // Scoring
      calculateScores: async (cardIds, champion, ring, covenant) => {
        const currentDeck = get().cards.map(c => c.id);
        
        try {
          const scores = await api.calculateDraftScores(cardIds, {
            currentDeck,
            champion,
            ringNumber: ring,
            covenant,
          });
          
          set({ scoredCards: scores });
        } catch (err) {
          console.error('Failed to calculate scores:', err);
        }
      },

      getCardScore: (cardId) => {
        return get().scoredCards.get(cardId);
      },

      // Cache management
      cacheCards: (cards) => {
        const cacheUpdates: CardCache = {};
        cards.forEach(card => {
          cacheUpdates[card.id] = card;
        });
        
        set({ cache: { ...get().cache, ...cacheUpdates } });
      },

      getCachedCard: (id) => {
        return get().cache[id];
      },

      clearCache: () => {
        set({ cache: {}, scoredCards: new Map() });
      },

      // Filter actions
      setSearchQuery: (searchQuery) => {
        set({ filters: { ...get().filters, searchQuery } });
        // Auto-search after a short delay could be implemented here
      },

      setSelectedClan: (selectedClan) => {
        set({ filters: { ...get().filters, selectedClan } });
      },

      setSelectedRarity: (selectedRarity) => {
        set({ filters: { ...get().filters, selectedRarity } });
      },

      setSelectedType: (selectedType) => {
        set({ filters: { ...get().filters, selectedType } });
      },

      setValueRange: (min, max) => {
        set({ 
          filters: { 
            ...get().filters, 
            minValue: min,
            maxValue: max,
          } 
        });
      },

      resetFilters: () => {
        set({ filters: { ...DEFAULT_FILTERS } });
      },

      // Error handling
      setError: (error) => set({ error }),
      clearError: () => set({ error: null }),

      // Computed
      getFilteredCards: () => {
        const { cards, filters } = get();
        
        return cards.filter(card => {
          const matchesSearch = !filters.searchQuery || 
            card.name.toLowerCase().includes(filters.searchQuery.toLowerCase()) ||
            card.keywords.some(k => k.toLowerCase().includes(filters.searchQuery.toLowerCase()));
          
          const matchesClan = filters.selectedClan === 'All' || card.clan === filters.selectedClan;
          const matchesRarity = filters.selectedRarity === 'All' || card.rarity === filters.selectedRarity;
          const matchesType = filters.selectedType === 'All' || card.cardType === filters.selectedType;
          
          const matchesMinValue = filters.minValue === null || card.baseValue >= filters.minValue;
          const matchesMaxValue = filters.maxValue === null || card.baseValue <= filters.maxValue;
          
          return matchesSearch && matchesClan && matchesRarity && matchesType && matchesMinValue && matchesMaxValue;
        });
      },

      getClans: () => {
        const clans = new Set(get().cards.map(c => c.clan));
        return ['All', ...Array.from(clans).sort()];
      },

      getRarities: () => {
        const rarities = new Set(get().cards.map(c => c.rarity));
        return ['All', ...Array.from(rarities).sort()];
      },

      getCardTypes: () => {
        const types = new Set(get().cards.map(c => c.cardType));
        return ['All', ...Array.from(types).sort()];
      },
    }),
    { name: 'mt2-card-store' }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectCards = (state: CardStore) => state.cards;

export const selectFilteredCards = (state: CardStore) => state.getFilteredCards();

export const selectCardFilters = (state: CardStore) => state.filters;

export const selectCardStatus = (state: CardStore) => ({
  isLoading: state.isLoading,
  isSearching: state.isSearching,
  error: state.error,
});

export const selectCardCache = (state: CardStore) => state.cache;
