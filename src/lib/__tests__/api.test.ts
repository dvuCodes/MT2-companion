import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  getCardByName,
  getCardsByClan,
  searchCards,
  getAllCards,
} from '../api';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('getCardByName', () => {
    it('should call invoke with correct parameters', async () => {
      const mockCard = {
        id: 'test-card',
        name: 'Test Card',
        clan: 'Banished',
        card_type: 'Spell',
        rarity: 'Common',
        cost: 1,
        base_value: 70,
        tempo_score: 6,
        value_score: 7,
        keywords: ['test'],
        description: 'Test',
        expansion: 'base',
      };

      vi.mocked(invoke).mockResolvedValue(mockCard);

      const result = await getCardByName('Test Card');

      expect(invoke).toHaveBeenCalledWith('get_card_by_name', { name: 'Test Card' });
      expect(result).toEqual(mockCard);
    });

    it('should return null when card not found', async () => {
      vi.mocked(invoke).mockResolvedValue(null);

      const result = await getCardByName('NonExistent');

      expect(result).toBeNull();
    });
  });

  describe('getCardsByClan', () => {
    it('should call invoke with correct parameters', async () => {
      const mockCards = [
        { id: '1', name: 'Card 1', clan: 'Banished' },
        { id: '2', name: 'Card 2', clan: 'Banished' },
      ];

      vi.mocked(invoke).mockResolvedValue(mockCards);

      const result = await getCardsByClan('Banished');

      expect(invoke).toHaveBeenCalledWith('get_cards_by_clan', { clan: 'Banished' });
      expect(result).toHaveLength(2);
    });
  });

  describe('searchCards', () => {
    it('should call invoke with correct parameters', async () => {
      const mockCards = [{ id: '1', name: 'Deadly Plunge' }];

      vi.mocked(invoke).mockResolvedValue(mockCards);

      const result = await searchCards('plunge');

      expect(invoke).toHaveBeenCalledWith('search_cards', { query: 'plunge' });
      expect(result).toHaveLength(1);
    });
  });

  describe('getAllCards', () => {
    it('should call invoke', async () => {
      const mockCards = [{ id: '1', name: 'Card 1' }];

      vi.mocked(invoke).mockResolvedValue(mockCards);

      const result = await getAllCards();

      expect(invoke).toHaveBeenCalledWith('get_all_cards', undefined);
      expect(result).toHaveLength(1);
    });
  });
});
