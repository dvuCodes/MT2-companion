import { useEffect, useCallback } from 'react';
import { Search, Loader2, AlertCircle, RefreshCw } from 'lucide-react';
import { useCardStore } from '../../stores';
import type { Card } from '../../types';

// Clan options
const CLAN_OPTIONS = [
  'All',
  'Banished',
  'Pyreborne',
  'Luna Coven',
  'Underlegion',
  'Lazarus League',
  'Melting Remnant',
  'Hellhorned',
  'Railforged',
  'Wurmkin',
];

const RARITY_OPTIONS = ['All', 'Champion', 'Rare', 'Uncommon', 'Common'];

export function CardList() {
  // Get state and actions from store
  const {
    cards,
    filters,
    isLoading,
    isSearching,
    error,
    lastFetched,
    fetchAllCards,
    searchCards,
    setSearchQuery,
    setSelectedClan,
    setSelectedRarity,
    resetFilters,
    clearError,
    getFilteredCards,
  } = useCardStore();

  // Get filtered cards using the store's computed getter
  const filteredCards = getFilteredCards();

  // Debounced search
  const handleSearchChange = useCallback((query: string) => {
    setSearchQuery(query);
    // The actual API search is triggered separately or we could debounce here
  }, [setSearchQuery]);

  // Trigger search when query changes
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (filters.searchQuery) {
        searchCards(filters.searchQuery);
      } else if (filters.searchQuery === '' && cards.length === 0) {
        fetchAllCards();
      }
    }, 300);

    return () => clearTimeout(timeoutId);
  }, [filters.searchQuery, searchCards, fetchAllCards, cards.length]);

  // Load cards on mount if not already loaded
  useEffect(() => {
    if (cards.length === 0 && !isLoading && !error) {
      fetchAllCards();
    }
  }, [cards.length, isLoading, error, fetchAllCards]);

  // Helper functions for styling
  const getRarityColor = (rarity: string): string => {
    switch (rarity) {
      case 'Champion': return 'text-purple-400';
      case 'Rare': return 'text-yellow-400';
      case 'Uncommon': return 'text-blue-400';
      default: return 'text-gray-400';
    }
  };

  const getValueColor = (value: number): string => {
    if (value >= 90) return 'text-yellow-400 font-bold';
    if (value >= 80) return 'text-gray-200';
    if (value >= 70) return 'text-orange-400';
    return 'text-gray-500';
  };

  const getCardTypeIcon = (cardType: string): string => {
    switch (cardType.toLowerCase()) {
      case 'unit': return '⚔️';
      case 'spell': return '✨';
      case 'equipment': return '🛡️';
      case 'artifact': return '🏺';
      default: return '🃏';
    }
  };

  // Format last fetched time
  const getLastFetchedText = () => {
    if (!lastFetched) return '';
    const seconds = Math.floor((Date.now() - lastFetched) / 1000);
    if (seconds < 60) return 'Just now';
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
    return `${Math.floor(seconds / 3600)}h ago`;
  };

  return (
    <div className="h-full flex flex-col">
      {/* Search and Filter Bar */}
      <div className="p-4 border-b border-gray-700 bg-gray-800/50">
        <div className="flex gap-4 mb-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" size={20} />
            <input
              type="text"
              placeholder="Search cards..."
              value={filters.searchQuery}
              onChange={(e) => handleSearchChange(e.target.value)}
              disabled={isLoading}
              className="w-full bg-gray-800 border border-gray-700 rounded pl-10 pr-4 py-2 text-white placeholder-gray-500 focus:border-yellow-500 focus:outline-none disabled:opacity-50"
            />
            {isSearching && (
              <Loader2 className="absolute right-3 top-1/2 transform -translate-y-1/2 text-yellow-400 animate-spin" size={18} />
            )}
          </div>
          
          <select
            value={filters.selectedClan}
            onChange={(e) => setSelectedClan(e.target.value)}
            disabled={isLoading}
            className="bg-gray-800 border border-gray-700 rounded px-4 py-2 focus:border-yellow-500 focus:outline-none disabled:opacity-50"
          >
            {CLAN_OPTIONS.map(clan => (
              <option key={clan} value={clan}>
                {clan === 'All' ? 'All Clans' : clan}
              </option>
            ))}
          </select>
          
          <select
            value={filters.selectedRarity}
            onChange={(e) => setSelectedRarity(e.target.value)}
            disabled={isLoading}
            className="bg-gray-800 border border-gray-700 rounded px-4 py-2 focus:border-yellow-500 focus:outline-none disabled:opacity-50"
          >
            {RARITY_OPTIONS.map(rarity => (
              <option key={rarity} value={rarity}>
                {rarity === 'All' ? 'All Rarities' : rarity}
              </option>
            ))}
          </select>
          
          <button
            onClick={resetFilters}
            disabled={isLoading}
            className="px-3 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm disabled:opacity-50 transition-colors"
            title="Reset filters"
          >
            <RefreshCw size={18} />
          </button>
        </div>

        {/* Status Bar */}
        <div className="flex justify-between items-center text-xs text-gray-500">
          <div className="flex items-center gap-4">
            <span>
              Showing <strong className="text-white">{filteredCards.length}</strong> of{' '}
              <strong className="text-white">{cards.length}</strong> cards
            </span>
            {lastFetched && (
              <span>Updated: {getLastFetchedText()}</span>
            )}
          </div>
          
          {error && (
            <button
              onClick={clearError}
              className="flex items-center gap-1 text-red-400 hover:text-red-300"
            >
              <AlertCircle size={14} />
              Clear error
            </button>
          )}
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-900/30 border-b border-red-700 px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-2 text-red-200 text-sm">
            <AlertCircle size={16} />
            <span>{error}</span>
          </div>
          <button
            onClick={() => fetchAllCards()}
            className="text-sm text-red-300 hover:text-red-200 underline"
          >
            Retry
          </button>
        </div>
      )}

      {/* Loading State */}
      {isLoading && cards.length === 0 && (
        <div className="flex-1 flex flex-col items-center justify-center text-gray-400">
          <Loader2 className="animate-spin mb-4" size={32} />
          <p>Loading card database...</p>
        </div>
      )}

      {/* Cards Grid */}
      {!isLoading || cards.length > 0 ? (
        <div className="flex-1 overflow-y-auto p-4">
          {filteredCards.length > 0 ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {filteredCards.map((card: Card) => (
                <CardItem 
                  key={card.id} 
                  card={card} 
                  getRarityColor={getRarityColor}
                  getValueColor={getValueColor}
                  getCardTypeIcon={getCardTypeIcon}
                />
              ))}
            </div>
          ) : (
            <div className="h-full flex flex-col items-center justify-center text-gray-500">
              <p className="text-lg mb-2">No cards found</p>
              <p className="text-sm">
                {filters.searchQuery || filters.selectedClan !== 'All' || filters.selectedRarity !== 'All'
                  ? 'Try adjusting your filters'
                  : 'The card database appears to be empty'}
              </p>
              {(filters.searchQuery || filters.selectedClan !== 'All' || filters.selectedRarity !== 'All') && (
                <button
                  onClick={resetFilters}
                  className="mt-4 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
                >
                  Clear Filters
                </button>
              )}
            </div>
          )}
        </div>
      ) : null}
    </div>
  );
}

// Card Item Component
interface CardItemProps {
  card: Card;
  getRarityColor: (rarity: string) => string;
  getValueColor: (value: number) => string;
  getCardTypeIcon: (type: string) => string;
}

function CardItem({ card, getRarityColor, getValueColor, getCardTypeIcon }: CardItemProps) {
  return (
    <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 hover:border-yellow-600 transition-colors group">
      {/* Header */}
      <div className="flex justify-between items-start mb-2">
        <h3 className="font-bold text-white truncate pr-2">{card.name}</h3>
        <span className={`text-sm ${getRarityColor(card.rarity)}`}>
          {card.rarity}
        </span>
      </div>
      
      {/* Meta Info */}
      <div className="text-sm text-gray-400 mb-3">
        <span className="mr-2">{getCardTypeIcon(card.cardType)}</span>
        {card.clan} • {card.cardType}
        {card.cost !== null && ` • ${card.cost} Ember`}
      </div>
      
      {/* Scores */}
      <div className="space-y-1 mb-3">
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Base Value:</span>
          <span className={`text-lg ${getValueColor(card.baseValue)}`}>
            {card.baseValue}
          </span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Tempo:</span>
          <span className="text-sm text-gray-300">{card.tempoScore}</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-sm text-gray-400">Value:</span>
          <span className="text-sm text-gray-300">{card.valueScore}</span>
        </div>
      </div>
      
      {/* Keywords */}
      <div className="flex flex-wrap gap-1">
        {card.keywords.slice(0, 4).map((keyword, idx) => (
          <span
            key={idx}
            className="text-xs bg-gray-700 text-gray-300 px-2 py-1 rounded"
          >
            {keyword}
          </span>
        ))}
        {card.keywords.length > 4 && (
          <span className="text-xs text-gray-500 px-1">
            +{card.keywords.length - 4}
          </span>
        )}
      </div>
      
      {/* Description (shown on hover/focus) */}
      {card.description && (
        <div className="mt-3 pt-3 border-t border-gray-700 text-xs text-gray-500 line-clamp-2">
          {card.description}
        </div>
      )}
    </div>
  );
}
