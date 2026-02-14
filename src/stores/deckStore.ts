/**
 * Deck Store - Manages the current draft deck state
 * 
 * Tracks:
 * - Current deck cards with draft metadata
 * - Champion selection and path
 * - Current ring and covenant level
 * - Draft analysis (synergies, warnings, etc.)
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { DeckCard, DeckAnalysis, SynergyCheck, Champion } from '../types';

// Champions list (local to avoid circular imports)
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
// Types
// ============================================================================

interface DeckState {
  // Champion info
  championId: string;
  championPath: string;
  
  // Draft progress
  currentRing: number;
  covenantLevel: number;
  
  // Deck cards
  cards: DeckCard[];
  
  // UI state
  isLoading: boolean;
  error: string | null;
  lastSaved: number | null;
}

interface DeckActions {
  // Champion actions
  setChampion: (championId: string) => void;
  setChampionPath: (path: string) => void;
  
  // Progress actions
  setCurrentRing: (ring: number) => void;
  setCovenantLevel: (level: number) => void;
  
  // Card actions
  addCard: (card: Omit<DeckCard, 'draftOrder' | 'ringNumber'>) => void;
  removeCard: (cardId: string) => void;
  clearDeck: () => void;
  
  // Batch operations
  setDeck: (cards: DeckCard[]) => void;
  importDeck: (cardIds: string[]) => Promise<void>;
  
  // Error handling
  setError: (error: string | null) => void;
  setLoading: (loading: boolean) => void;
}

// Define the store type
interface DeckStore extends DeckState, DeckActions {
  // Computed properties (via selectors)
  getAnalysis: () => DeckAnalysis;
  getUnits: () => DeckCard[];
  getSpells: () => DeckCard[];
  getCardCount: () => number;
}

// ============================================================================
// Synergy Definitions
// ============================================================================

const SYNERGIES: SynergyCheck[] = [
  { name: 'Shift Loop', description: 'Shift + Advance cards', active: false },
  { name: 'Consume Chain', description: 'Consume + Funguy synergy', active: false },
  { name: 'Reform Engine', description: 'Reform + Burnout synergy', active: false },
  { name: 'Dragon Hoard', description: 'Dragon Hoard scaling', active: false },
  { name: 'Forge Burn', description: 'Forge + Smelt synergy', active: false },
];

// ============================================================================
// Store
// ============================================================================

export const useDeckStore = create<DeckStore>()(
  persist(
    (set, get) => ({
      // Initial state
      championId: 'fel',
      championPath: 'Unchained',
      currentRing: 1,
      covenantLevel: 10,
      cards: [],
      isLoading: false,
      error: null,
      lastSaved: null,

      // Champion actions
      setChampion: (championId) => {
        const champion = CHAMPIONS.find(c => c.id === championId);
        set({ 
          championId,
          // Reset path to first available if current is invalid
          championPath: champion?.paths[0] || 'Unchained',
        });
      },

      setChampionPath: (championPath) => set({ championPath }),

      // Progress actions
      setCurrentRing: (currentRing) => set({ currentRing }),
      
      setCovenantLevel: (covenantLevel) => set({ covenantLevel }),

      // Card actions
      addCard: (card) => {
        const state = get();
        const draftOrder = state.cards.length + 1;
        
        const deckCard: DeckCard = {
          ...card,
          draftOrder,
          ringNumber: state.currentRing,
        };
        
        set({ 
          cards: [...state.cards, deckCard],
          lastSaved: Date.now(),
        });
      },

      removeCard: (cardId) => {
        const state = get();
        const filtered = state.cards.filter(c => c.id !== cardId);
        // Recalculate draft order
        const reordered = filtered.map((c, idx) => ({
          ...c,
          draftOrder: idx + 1,
        }));
        
        set({ 
          cards: reordered,
          lastSaved: Date.now(),
        });
      },

      clearDeck: () => set({ 
        cards: [],
        currentRing: 1,
        lastSaved: Date.now(),
      }),

      // Batch operations
      setDeck: (cards) => set({ 
        cards,
        lastSaved: Date.now(),
      }),

      importDeck: async (cardIds) => {
        set({ isLoading: true, error: null });
        try {
          // In a real implementation, this would fetch card data from the backend
          // For now, we'll create placeholder cards
          // const { championId, currentRing } = get(); // Reserved for future use
          
          const deckCards: DeckCard[] = cardIds.map((id, idx) => ({
            id,
            name: id, // Would be fetched from backend
            clan: 'Unknown',
            cardType: 'Unknown',
            rarity: 'Common',
            cost: null,
            baseValue: 0,
            tempoScore: 0,
            valueScore: 0,
            keywords: [],
            description: '',
            expansion: '',
            draftOrder: idx + 1,
            ringNumber: Math.floor(idx / 3) + 1,
          }));
          
          set({ 
            cards: deckCards,
            isLoading: false,
            lastSaved: Date.now(),
          });
        } catch (err) {
          set({ 
            error: err instanceof Error ? err.message : 'Failed to import deck',
            isLoading: false,
          });
        }
      },

      // Error handling
      setError: (error) => set({ error }),
      setLoading: (isLoading) => set({ isLoading }),

      // Computed getters
      getAnalysis: () => {
        const { cards } = get();
        
        const allKeywords = cards.flatMap(c => c.keywords);
        
        // Check for critical components
        const hasFrontline = allKeywords.some(k => 
          ['frontline', 'tank', 'armor'].includes(k.toLowerCase())
        );
        const hasBacklineClear = allKeywords.some(k =>
          ['sweep', 'explosive', 'advance', 'aoe'].includes(k.toLowerCase())
        );
        const hasScaling = allKeywords.some(k =>
          ['valor', 'decay', 'conduit', 'pyregel', 'dragon_hoard', 'forge'].includes(k.toLowerCase())
        );

        // Calculate synergies
        const activeSynergies = SYNERGIES.map(synergy => {
          let active = false;
          
          switch (synergy.name) {
            case 'Shift Loop':
              active = allKeywords.includes('shift') && allKeywords.includes('advance');
              break;
            case 'Consume Chain':
              active = allKeywords.includes('consume') && allKeywords.includes('funguy');
              break;
            case 'Reform Engine':
              active = allKeywords.includes('reform') && allKeywords.includes('burnout');
              break;
            case 'Dragon Hoard':
              active = allKeywords.includes('dragon_hoard');
              break;
            case 'Forge Burn':
              active = allKeywords.includes('forge') && allKeywords.includes('smelt');
              break;
          }
          
          return { ...synergy, active };
        });

        const units = cards.filter(c => c.cardType.toLowerCase() === 'unit');
        const spells = cards.filter(c => c.cardType.toLowerCase() === 'spell');
        
        const totalValue = cards.reduce((sum, c) => sum + c.baseValue, 0);
        const averageValue = cards.length > 0 ? totalValue / cards.length : 0;

        return {
          hasFrontline,
          hasBacklineClear,
          hasScaling,
          activeSynergies,
          totalCards: cards.length,
          unitCount: units.length,
          spellCount: spells.length,
          averageValue: Math.round(averageValue),
        };
      },

      getUnits: () => {
        return get().cards.filter(c => c.cardType.toLowerCase() === 'unit');
      },

      getSpells: () => {
        return get().cards.filter(c => c.cardType.toLowerCase() === 'spell');
      },

      getCardCount: () => {
        return get().cards.length;
      },
    }),
    {
      name: 'mt2-draft-deck',
      partialize: (state) => ({
        championId: state.championId,
        championPath: state.championPath,
        currentRing: state.currentRing,
        covenantLevel: state.covenantLevel,
        cards: state.cards,
      }),
    }
  )
);

// ============================================================================
// Selectors (for optimized re-renders)
// ============================================================================

export const selectChampion = (state: DeckStore) => ({
  championId: state.championId,
  championPath: state.championPath,
});

export const selectDraftProgress = (state: DeckStore) => ({
  currentRing: state.currentRing,
  covenantLevel: state.covenantLevel,
});

export const selectDeckCards = (state: DeckStore) => state.cards;

export const selectDeckStatus = (state: DeckStore) => ({
  isLoading: state.isLoading,
  error: state.error,
});
