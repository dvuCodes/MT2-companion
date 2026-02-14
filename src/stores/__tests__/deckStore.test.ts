import { describe, it, expect, beforeEach } from 'vitest';
import { useDeckStore } from '../deckStore';

describe('deckStore', () => {
  beforeEach(() => {
    const store = useDeckStore.getState();
    store.clearDeck();
    store.setChampion('fel');
    store.setCovenantLevel(10);
    store.setCurrentRing(1);
  });

  it('should initialize with default values', () => {
    const state = useDeckStore.getState();
    expect(state.championId).toBe('fel');
    expect(state.championPath).toBe('Unchained');
    expect(state.covenantLevel).toBe(10);
    expect(state.currentRing).toBe(1);
    expect(state.cards).toEqual([]);
  });

  it('should add cards to deck', () => {
    const store = useDeckStore.getState();
    const card = {
      id: 'test-card',
      name: 'Test Card',
      clan: 'Banished',
      cardType: 'Spell',
      rarity: 'Common',
      cost: 1,
      baseValue: 70,
      tempoScore: 6,
      valueScore: 7,
      keywords: ['test'],
      description: 'Test card',
      expansion: 'base',
    };

    store.addCard(card);

    expect(useDeckStore.getState().cards).toHaveLength(1);
    expect(useDeckStore.getState().cards[0].name).toBe('Test Card');
  });

  it('should remove cards from deck', () => {
    const store = useDeckStore.getState();
    const card = {
      id: 'test-card',
      name: 'Test Card',
      clan: 'Banished',
      cardType: 'Spell',
      rarity: 'Common',
      cost: 1,
      baseValue: 70,
      tempoScore: 6,
      valueScore: 7,
      keywords: ['test'],
      description: 'Test card',
      expansion: 'base',
    };

    store.addCard(card);
    store.removeCard('test-card');

    expect(useDeckStore.getState().cards).toHaveLength(0);
  });

  it('should detect missing frontline', () => {
    const store = useDeckStore.getState();
    expect(store.getAnalysis().hasFrontline).toBe(false);
  });

  it('should detect frontline when tank card added', () => {
    const store = useDeckStore.getState();
    const tankCard = {
      id: 'tank-card',
      name: 'Tank Card',
      clan: 'Banished',
      cardType: 'Unit',
      rarity: 'Common',
      cost: 2,
      baseValue: 75,
      tempoScore: 5,
      valueScore: 8,
      keywords: ['frontline', 'tank'],
      description: 'A tank card',
      expansion: 'base',
    };

    store.addCard(tankCard);

    expect(useDeckStore.getState().getAnalysis().hasFrontline).toBe(true);
  });

  it('should update champion', () => {
    const store = useDeckStore.getState();
    store.setChampion('talos');

    expect(useDeckStore.getState().championId).toBe('talos');
  });

  it('should update ring number', () => {
    const store = useDeckStore.getState();
    store.setCurrentRing(5);

    expect(useDeckStore.getState().currentRing).toBe(5);
  });
});
