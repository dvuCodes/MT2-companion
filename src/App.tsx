import { useState, useEffect } from 'react';
import { DeckTracker } from './components/deck-tracker/DeckTracker';
import { DraftOverlay } from './components/draft-overlay/DraftOverlay';
import { CardList } from './components/card-display/CardList';
import { useCardStore, useSettingsStore } from './stores';
import { Loader2 } from 'lucide-react';

type Tab = 'deck' | 'cards' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('deck');
  
  // Global stores
  const { fetchAllCards, isLoading: isLoadingCards, error: cardsError } = useCardStore();
  const { covenantLevel, ocrMode, setCovenantLevel, setOcrMode } = useSettingsStore();

  // Load cards on mount
  useEffect(() => {
    fetchAllCards();
  }, [fetchAllCards]);

  return (
    <div className="h-screen flex flex-col bg-gray-900 text-white">
      {/* Header */}
      <header className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex justify-between items-center">
        <h1 className="text-xl font-bold text-yellow-400">MT2 Draft Assistant</h1>
        
        <div className="flex gap-2">
          <button
            onClick={() => setActiveTab('deck')}
            className={`px-4 py-2 rounded transition-colors ${
              activeTab === 'deck' ? 'bg-yellow-600' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            Deck
          </button>
          <button
            onClick={() => setActiveTab('cards')}
            className={`px-4 py-2 rounded transition-colors ${
              activeTab === 'cards' ? 'bg-yellow-600' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            Cards
          </button>
          <button
            onClick={() => setActiveTab('settings')}
            className={`px-4 py-2 rounded transition-colors ${
              activeTab === 'settings' ? 'bg-yellow-600' : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            Settings
          </button>
        </div>
      </header>

      {/* Error Banner */}
      {cardsError && (
        <div className="bg-red-900/50 border-b border-red-700 px-4 py-2 text-red-200 text-sm">
          Error loading cards: {cardsError}
        </div>
      )}

      {/* Loading Overlay */}
      {isLoadingCards && activeTab === 'cards' && (
        <div className="absolute inset-0 bg-gray-900/80 flex items-center justify-center z-40">
          <div className="flex items-center gap-3 text-yellow-400">
            <Loader2 className="animate-spin" size={24} />
            <span>Loading cards...</span>
          </div>
        </div>
      )}

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        {activeTab === 'deck' && (
          <div className="h-full flex">
            <DeckTracker />
            <div className="flex-1 p-4">
              <div className="text-center text-gray-400 mt-20">
                <p className="text-lg">Start a draft to see recommendations</p>
                <p className="text-sm mt-2">Use the overlay window during gameplay</p>
                <button
                  onClick={() => useCardStore.getState().fetchAllCards()}
                  className="mt-4 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm"
                  disabled={isLoadingCards}
                >
                  {isLoadingCards ? 'Loading...' : 'Refresh Card Database'}
                </button>
              </div>
            </div>
          </div>
        )}
        
        {activeTab === 'cards' && <CardList />}
        
        {activeTab === 'settings' && (
          <div className="p-4 max-w-2xl">
            <h2 className="text-lg font-bold mb-4">Settings</h2>
            
            <div className="space-y-6">
              {/* OCR Settings */}
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-sm font-bold text-gray-400 uppercase mb-3">OCR & Detection</h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">OCR Mode</label>
                    <select
                      value={ocrMode}
                      onChange={(e) => setOcrMode(e.target.value as 'automatic' | 'manual' | 'hybrid')}
                      className="w-full max-w-xs bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:border-yellow-500 focus:outline-none"
                    >
                      <option value="automatic">Automatic (OCR)</option>
                      <option value="manual">Manual Input</option>
                      <option value="hybrid">Hybrid</option>
                    </select>
                    <p className="text-xs text-gray-500 mt-1">
                      How cards are detected during drafting
                    </p>
                  </div>
                </div>
              </div>
              
              {/* Draft Settings */}
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-sm font-bold text-gray-400 uppercase mb-3">Draft Settings</h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-1">Default Covenant Level</label>
                    <div className="flex items-center gap-2">
                      <input
                        type="number"
                        min="0"
                        max="25"
                        value={covenantLevel}
                        onChange={(e) => setCovenantLevel(parseInt(e.target.value) || 0)}
                        className="w-24 bg-gray-700 border border-gray-600 rounded px-3 py-2 focus:border-yellow-500 focus:outline-none"
                      />
                      <span className="text-gray-400 text-sm">(0-25)</span>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">
                      Used for scoring calculations
                    </p>
                  </div>
                </div>
              </div>
              
              {/* Database Info */}
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-sm font-bold text-gray-400 uppercase mb-3">Database</h3>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-gray-400">
                    Card database status
                  </span>
                  <span className={`text-sm ${cardsError ? 'text-red-400' : 'text-green-400'}`}>
                    {isLoadingCards ? 'Loading...' : cardsError ? 'Error' : 'Connected'}
                  </span>
                </div>
                <button
                  onClick={() => fetchAllCards()}
                  disabled={isLoadingCards}
                  className="mt-3 px-3 py-1.5 bg-yellow-600 hover:bg-yellow-500 disabled:bg-gray-600 disabled:cursor-not-allowed rounded text-sm transition-colors"
                >
                  {isLoadingCards ? 'Refreshing...' : 'Refresh Cards'}
                </button>
              </div>
              
              {/* Keyboard Shortcuts */}
              <div className="bg-gray-800 rounded-lg p-4">
                <h3 className="text-sm font-bold text-gray-400 uppercase mb-3">Keyboard Shortcuts</h3>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-400">Toggle Overlay</span>
                    <kbd className="bg-gray-700 px-2 py-0.5 rounded text-xs">Ctrl + Shift + O</kbd>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-400">Detect Cards</span>
                    <kbd className="bg-gray-700 px-2 py-0.5 rounded text-xs">Ctrl + Shift + D</kbd>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-400">Calibrate OCR</span>
                    <kbd className="bg-gray-700 px-2 py-0.5 rounded text-xs">Ctrl + Shift + C</kbd>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </main>

      {/* Draft Overlay */}
      <DraftOverlay />
    </div>
  );
}

export default App;
