import { useEffect, useCallback } from 'react';
import { useOverlayStore, useDeckStore, useCardStore, CHAMPIONS } from '../../stores';
import { Loader2, AlertCircle, RefreshCw, X } from 'lucide-react';
import type { ScoredCard } from '../../types';

export function DraftOverlay() {
  // Get state from stores
  const {
    isVisible,
    detectedCards,
    isDetecting,
    isScoring,
    detectionError,
    scoringError,
    show,
    hide,
    toggle,
    detectCards,
    calculateScores,
    clearErrors,
    getScoredCards,

  } = useOverlayStore();

  const {
    championId,
    championPath,
    currentRing,
    covenantLevel,
    cards: deckCards,
  } = useDeckStore();

  useCardStore(); // Ensure card store is initialized

  // Get scored cards
  const scoredCards = getScoredCards();

  // Calculate scores when detected cards change
  useEffect(() => {
    if (detectedCards.length > 0 && isVisible) {
      calculateScores(
        championId,
        currentRing,
        covenantLevel,
        deckCards.map(c => c.id)
      );
    }
  }, [detectedCards, championId, currentRing, covenantLevel, deckCards, calculateScores, isVisible]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl/Cmd + Shift + O: Toggle overlay
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'O') {
        e.preventDefault();
        toggle();
      }
      // Ctrl/Cmd + Shift + D: Detect cards
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'D') {
        e.preventDefault();
        handleDetect();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [toggle]);

  // Handle card detection
  const handleDetect = useCallback(async () => {
    await detectCards();
  }, [detectCards]);

  // Handle card selection
  const handleSelectCard = useCallback((cardName: string) => {
    // This would log the selection or trigger an action
    console.log(`Selected: ${cardName}`);
    // Could also add to deck here if desired
  }, []);

  // Get tier color for styling
  const getTierColor = (tier: string): string => {
    switch (tier.toUpperCase()) {
      case 'S': return 'text-yellow-400 border-yellow-400';
      case 'A': return 'text-gray-300 border-gray-300';
      case 'B': return 'text-orange-400 border-orange-400';
      case 'C': return 'text-gray-500 border-gray-500';
      case 'D': return 'text-red-400 border-red-400';
      default: return 'text-gray-500 border-gray-500';
    }
  };

  const getScoreColor = (score: number): string => {
    if (score >= 90) return 'text-yellow-400';
    if (score >= 80) return 'text-gray-200';
    if (score >= 70) return 'text-orange-400';
    return 'text-gray-500';
  };

  // Toggle button when hidden
  if (!isVisible) {
    return (
      <button
        onClick={() => show()}
        className="fixed bottom-4 right-4 bg-yellow-600 hover:bg-yellow-500 text-white px-4 py-2 rounded shadow-lg z-50 flex items-center gap-2 transition-colors"
      >
        Show Overlay
      </button>
    );
  }

  return (
    <div className="fixed inset-0 pointer-events-none z-50">
      {/* Overlay Window Content */}
      <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 pointer-events-auto w-full max-w-4xl">
        
        {/* Error Banner */}
        {(detectionError || scoringError) && (
          <div className="mb-4 bg-red-900/90 border border-red-700 rounded-lg p-3 text-red-200 text-sm flex items-center justify-between">
            <div className="flex items-center gap-2">
              <AlertCircle size={16} />
              <span>{detectionError || scoringError}</span>
            </div>
            <button
              onClick={clearErrors}
              className="text-red-300 hover:text-red-100"
            >
              <X size={16} />
            </button>
          </div>
        )}

        {/* Loading State */}
        {(isDetecting || isScoring) && detectedCards.length === 0 && (
          <div className="flex justify-center mb-4">
            <div className="bg-gray-900/95 border border-gray-700 rounded-lg p-6 flex items-center gap-3 text-yellow-400">
              <Loader2 className="animate-spin" size={24} />
              <span>{isDetecting ? 'Detecting cards...' : 'Calculating scores...'}</span>
            </div>
          </div>
        )}

        {/* Cards Display */}
        {scoredCards.length > 0 ? (
          <div className="flex justify-center gap-4">
            {scoredCards.map((card: ScoredCard) => (
              <CardScoreDisplay
                key={card.id}
                card={card}
                getTierColor={getTierColor}
                getScoreColor={getScoreColor}
                onSelect={() => handleSelectCard(card.name)}
              />
            ))}
          </div>
        ) : detectedCards.length === 0 && !isDetecting ? (
          <div className="flex justify-center">
            <div className="bg-gray-900/95 border border-gray-700 rounded-lg p-6 text-center">
              <p className="text-gray-400 mb-4">No cards detected</p>
              <button
                onClick={handleDetect}
                disabled={isDetecting}
                className="flex items-center gap-2 px-4 py-2 bg-yellow-600 hover:bg-yellow-500 disabled:bg-gray-600 rounded transition-colors"
              >
                <RefreshCw size={16} className={isDetecting ? 'animate-spin' : ''} />
                {isDetecting ? 'Detecting...' : 'Detect Cards'}
              </button>
            </div>
          </div>
        ) : null}

        {/* Controls */}
        <div className="absolute -bottom-12 left-1/2 transform -translate-x-1/2 flex items-center gap-2">
          <button
            onClick={handleDetect}
            disabled={isDetecting}
            className="bg-gray-800 hover:bg-gray-700 text-white px-3 py-1.5 rounded text-sm flex items-center gap-1.5 transition-colors disabled:opacity-50"
          >
            <RefreshCw size={14} className={isDetecting ? 'animate-spin' : ''} />
            {isDetecting ? 'Detecting...' : 'Detect'}
          </button>
          
          <button
            onClick={() => hide()}
            className="bg-gray-800 hover:bg-gray-700 text-white px-3 py-1.5 rounded text-sm transition-colors"
          >
            Hide Overlay
          </button>
        </div>

        {/* Draft Context Info */}
        <div className="absolute -top-8 left-1/2 transform -translate-x-1/2 bg-gray-900/90 border border-gray-700 rounded px-3 py-1 text-xs text-gray-400">
          {CHAMPIONS.find(c => c.id === championId)?.name} • {championPath} • Ring {currentRing} • Covenant {covenantLevel}
        </div>
      </div>
    </div>
  );
}

// Card Score Display Component
interface CardScoreDisplayProps {
  card: ScoredCard;
  getTierColor: (tier: string) => string;
  getScoreColor: (score: number) => string;
  onSelect: () => void;
}

function CardScoreDisplay({ card, getTierColor, getScoreColor, onSelect }: CardScoreDisplayProps) {
  return (
    <div
      className={`bg-gray-900/95 border-2 rounded-lg p-4 w-52 text-center backdrop-blur-sm ${getTierColor(card.tier)}`}
    >
      {/* Score Badge */}
      <div className={`text-4xl font-bold mb-2 ${getScoreColor(card.score)}`}>
        {card.score}
      </div>
      
      {/* Tier Badge */}
      <div className={`text-lg font-bold mb-1 ${getTierColor(card.tier)}`}>
        {card.tier === 'S' && '★ '}
        {card.tier}-Tier
      </div>
      
      {/* Card Name */}
      <div className="text-white text-sm font-medium mb-2 truncate" title={card.name}>
        {card.name}
      </div>
      
      {/* Card Info */}
      <div className="text-xs text-gray-500 mb-2">
        {card.clan} • {card.cardType}
        {card.cost !== null && ` • ${card.cost}E`}
      </div>
      
      {/* Reasons */}
      <div className="text-xs text-gray-400 space-y-1 mb-3">
        {card.reasons.slice(0, 3).map((reason, ridx) => (
          <div key={ridx} className="truncate" title={reason}>{reason}</div>
        ))}
        {card.reasons.length > 3 && (
          <div className="text-gray-600">+{card.reasons.length - 3} more</div>
        )}
      </div>
      
      {/* Keywords */}
      {card.keywords.length > 0 && (
        <div className="flex flex-wrap justify-center gap-1 mb-3">
          {card.keywords.slice(0, 3).map((keyword, kidx) => (
            <span key={kidx} className="text-[10px] bg-gray-800 text-gray-400 px-1.5 py-0.5 rounded">
              {keyword}
            </span>
          ))}
        </div>
      )}
      
      {/* Select Button */}
      <button
        onClick={onSelect}
        className="w-full bg-yellow-600 hover:bg-yellow-500 text-white text-sm py-1.5 px-2 rounded transition-colors"
      >
        Select
      </button>
    </div>
  );
}


