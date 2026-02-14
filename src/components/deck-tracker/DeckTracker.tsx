import { useDeckStore, CHAMPIONS } from '../../stores';
import { Shield, Sword, AlertTriangle, CheckCircle, Zap, Loader2 } from 'lucide-react';

export function DeckTracker() {
  // Get state and actions from store
  const {
    championId,
    championPath,
    currentRing,
    covenantLevel,
    cards,
    isLoading,
    error,
    setChampion,
    setChampionPath,
    setCurrentRing,
    setCovenantLevel,
    clearDeck,
    getAnalysis,
    getUnits,
    getSpells,
  } = useDeckStore();

  // Get computed analysis
  const analysis = getAnalysis();
  const units = getUnits();
  const spells = getSpells();

  // Get current champion info
  const currentChampion = CHAMPIONS.find(c => c.id === championId);
  const availablePaths = currentChampion?.paths || ['Unchained', 'Savior'];

  return (
    <div className="w-80 h-full bg-gray-800 border-r border-gray-700 flex flex-col">
      {/* Error Display */}
      {error && (
        <div className="bg-red-900/50 border-b border-red-700 px-4 py-2 text-red-200 text-xs">
          {error}
        </div>
      )}

      {/* Loading Indicator */}
      {isLoading && (
        <div className="flex items-center justify-center gap-2 py-2 text-yellow-400 text-sm">
          <Loader2 className="animate-spin" size={16} />
          <span>Loading...</span>
        </div>
      )}

      {/* Champion Section */}
      <div className="p-4 border-b border-gray-700">
        <h3 className="text-sm font-bold text-gray-400 uppercase mb-2">Champion</h3>
        
        <select
          value={championId}
          onChange={(e) => setChampion(e.target.value)}
          disabled={isLoading}
          className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 mb-2 disabled:opacity-50 focus:border-yellow-500 focus:outline-none"
        >
          {CHAMPIONS.map(champion => (
            <option key={champion.id} value={champion.id}>
              {champion.name}
            </option>
          ))}
        </select>
        
        <select
          value={championPath}
          onChange={(e) => setChampionPath(e.target.value)}
          disabled={isLoading}
          className="w-full bg-gray-700 border border-gray-600 rounded px-3 py-2 mb-2 text-sm disabled:opacity-50 focus:border-yellow-500 focus:outline-none"
        >
          {availablePaths.map(path => (
            <option key={path} value={path}>
              {path}
            </option>
          ))}
        </select>
        
        <div className="flex gap-2 mt-2">
          <div className="flex-1">
            <label className="text-xs text-gray-500">Ring</label>
            <input
              type="number"
              min="1"
              max="9"
              value={currentRing}
              onChange={(e) => setCurrentRing(parseInt(e.target.value) || 1)}
              disabled={isLoading}
              className="w-full bg-gray-700 border border-gray-600 rounded px-2 py-1 text-sm disabled:opacity-50 focus:border-yellow-500 focus:outline-none"
            />
          </div>
          <div className="flex-1">
            <label className="text-xs text-gray-500">Covenant</label>
            <input
              type="number"
              min="0"
              max="25"
              value={covenantLevel}
              onChange={(e) => setCovenantLevel(parseInt(e.target.value) || 0)}
              disabled={isLoading}
              className="w-full bg-gray-700 border border-gray-600 rounded px-2 py-1 text-sm disabled:opacity-50 focus:border-yellow-500 focus:outline-none"
            />
          </div>
        </div>
      </div>

      {/* Stats Summary */}
      <div className="px-4 py-2 border-b border-gray-700 bg-gray-800/50">
        <div className="flex justify-between text-xs text-gray-400">
          <span>Total: <strong className="text-white">{analysis.totalCards}</strong></span>
          <span>Avg Value: <strong className="text-white">{analysis.averageValue}</strong></span>
        </div>
      </div>

      {/* Warnings */}
      <div className="p-4 border-b border-gray-700">
        <h3 className="text-sm font-bold text-gray-400 uppercase mb-2">Warnings</h3>
        <div className="space-y-2">
          {!analysis.hasFrontline && (
            <div className="flex items-center gap-2 text-red-400">
              <AlertTriangle size={16} />
              <span className="text-sm">No Frontline</span>
            </div>
          )}
          {!analysis.hasBacklineClear && (
            <div className="flex items-center gap-2 text-orange-400">
              <AlertTriangle size={16} />
              <span className="text-sm">No Backline Clear</span>
            </div>
          )}
          {!analysis.hasScaling && currentRing >= 5 && (
            <div className="flex items-center gap-2 text-yellow-400">
              <AlertTriangle size={16} />
              <span className="text-sm">No Scaling (Ring 5+)</span>
            </div>
          )}
          {analysis.hasFrontline && analysis.hasBacklineClear && (
            <div className="flex items-center gap-2 text-green-400">
              <CheckCircle size={16} />
              <span className="text-sm">Core Needs Met</span>
            </div>
          )}
          {analysis.totalCards === 0 && (
            <div className="text-sm text-gray-500 italic">
              Start drafting to see analysis
            </div>
          )}
        </div>
      </div>

      {/* Active Synergies */}
      <div className="p-4 border-b border-gray-700">
        <h3 className="text-sm font-bold text-gray-400 uppercase mb-2">Synergies</h3>
        <div className="space-y-1">
          {analysis.activeSynergies.map((synergy, idx) => (
            <div
              key={idx}
              className={`flex items-center gap-2 text-sm ${
                synergy.active ? 'text-green-400' : 'text-gray-600'
              }`}
              title={synergy.description}
            >
              {synergy.active ? <Zap size={14} /> : <span className="w-3.5"></span>}
              <span>{synergy.name}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Deck List */}
      <div className="flex-1 overflow-y-auto">
        {/* Units Section */}
        <div className="p-4 border-b border-gray-700">
          <h3 className="text-sm font-bold text-blue-400 uppercase mb-2 flex items-center gap-2">
            <Shield size={16} />
            Units ({units.length})
          </h3>
          
          {units.length === 0 ? (
            <p className="text-sm text-gray-600 italic">No units drafted</p>
          ) : (
            <div className="space-y-1">
              {units.map((card) => (
                <div 
                  key={card.id} 
                  className="text-sm bg-gray-700 px-2 py-1 rounded flex justify-between items-center group"
                >
                  <span className="truncate">{card.name}</span>
                  <span className="text-xs text-gray-500 ml-2">R{card.ringNumber}</span>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Spells Section */}
        <div className="p-4">
          <h3 className="text-sm font-bold text-red-400 uppercase mb-2 flex items-center gap-2">
            <Sword size={16} />
            Spells ({spells.length})
          </h3>
          
          {spells.length === 0 ? (
            <p className="text-sm text-gray-600 italic">No spells drafted</p>
          ) : (
            <div className="space-y-1">
              {spells.map((card) => (
                <div 
                  key={card.id} 
                  className="text-sm bg-gray-700 px-2 py-1 rounded flex justify-between items-center group"
                >
                  <span className="truncate">{card.name}</span>
                  <span className="text-xs text-gray-500 ml-2">R{card.ringNumber}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Clear Deck Button */}
      {cards.length > 0 && (
        <div className="p-4 border-t border-gray-700">
          <button
            onClick={clearDeck}
            disabled={isLoading}
            className="w-full py-2 px-4 bg-red-900/50 hover:bg-red-800/50 text-red-300 rounded text-sm transition-colors disabled:opacity-50"
          >
            Clear Deck
          </button>
        </div>
      )}
    </div>
  );
}
