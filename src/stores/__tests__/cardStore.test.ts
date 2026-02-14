import { describe, it, expect, beforeEach } from 'vitest';
import { useCardStore } from '../cardStore';

describe('cardStore', () => {
  beforeEach(() => {
    const store = useCardStore.getState();
    store.clearCache();
  });

  it('should initialize with empty cache', () => {
    const state = useCardStore.getState();
    expect(Object.keys(state.cache)).toHaveLength(0);
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it('should cache cards', () => {
    const store = useCardStore.getState();
    const mockCards = [
      {
        id: 'card-1',
        name: 'Card One',
        clan: 'Banished',
        cardType: 'Spell',
        rarity: 'Common',
        cost: 1,
        baseValue: 70,
        tempoScore: 6,
        valueScore: 7,
        keywords: ['test'],
        description: 'Test card 1',
        expansion: 'base',
      },
    ];

    store.cacheCards(mockCards);

    expect(Object.keys(useCardStore.getState().cache)).toHaveLength(1);
    expect(useCardStore.getState().cache['card-1'].name).toBe('Card One');
  });

  it('should get cached card by id', () => {
    const store = useCardStore.getState();
    const mockCards = [
      {
        id: 'card-1',
        name: 'Test Card',
        clan: 'Banished',
        cardType: 'Spell',
        rarity: 'Common',
        cost: 1,
        baseValue: 70,
        tempoScore: 6,
        valueScore: 7,
        keywords: [],
        description: 'Test card',
        expansion: 'base',
      },
    ];

    store.cacheCards(mockCards);

    const card = useCardStore.getState().getCachedCard('card-1');
    expect(card).not.toBeUndefined();
    expect(card?.name).toBe('Test Card');
  });

  it('should return undefined for non-existent card', () => {
    const store = useCardStore.getState();
    store.cacheCards([]);

    const card = useCardStore.getState().getCachedCard('non-existent');
    expect(card).toBeUndefined();
  });
});
