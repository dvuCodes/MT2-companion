/**
 * Settings Store - Manages application settings
 * 
 * Persists user preferences to localStorage.
 * Settings are synced across app sessions.
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { AppSettings, OverlayMode } from '../types';
import { DEFAULT_SETTINGS } from '../types';

// ============================================================================
// Types
// ============================================================================

interface SettingsState extends AppSettings {
  // UI state for settings panel
  isSettingsOpen: boolean;
  hasUnsavedChanges: boolean;
}

interface SettingsActions {
  // Settings updates
  setOcrMode: (mode: OverlayMode) => void;
  setCovenantLevel: (level: number) => void;
  setOverlayOpacity: (opacity: number) => void;
  setShowCardArt: (show: boolean) => void;
  setEnableSoundAlerts: (enable: boolean) => void;
  setTheme: (theme: AppSettings['theme']) => void;
  
  // Batch update
  updateSettings: (settings: Partial<AppSettings>) => void;
  
  // Reset
  resetToDefaults: () => void;
  
  // UI state
  openSettings: () => void;
  closeSettings: () => void;
  toggleSettings: () => void;
  markSaved: () => void;
}

interface SettingsStore extends SettingsState, SettingsActions {}

// ============================================================================
// Validation Helpers
// ============================================================================

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

// ============================================================================
// Store
// ============================================================================

export const useSettingsStore = create<SettingsStore>()(
  persist(
    (set, get) => ({
      // Initial state from defaults
      ...DEFAULT_SETTINGS,
      
      // UI state
      isSettingsOpen: false,
      hasUnsavedChanges: false,

      // Individual setting updates
      setOcrMode: (ocrMode) => {
        set({ 
          ocrMode,
          hasUnsavedChanges: true,
        });
      },

      setCovenantLevel: (covenantLevel) => {
        set({ 
          covenantLevel: clamp(covenantLevel, 0, 25),
          hasUnsavedChanges: true,
        });
      },

      setOverlayOpacity: (overlayOpacity) => {
        set({ 
          overlayOpacity: clamp(overlayOpacity, 0.1, 1),
          hasUnsavedChanges: true,
        });
      },

      setShowCardArt: (showCardArt) => {
        set({ 
          showCardArt,
          hasUnsavedChanges: true,
        });
      },

      setEnableSoundAlerts: (enableSoundAlerts) => {
        set({ 
          enableSoundAlerts,
          hasUnsavedChanges: true,
        });
      },

      setTheme: (theme) => {
        set({ 
          theme,
          hasUnsavedChanges: true,
        });
      },

      // Batch update
      updateSettings: (settings) => {
        set({ 
          ...settings,
          // Ensure values are clamped
          covenantLevel: settings.covenantLevel !== undefined 
            ? clamp(settings.covenantLevel, 0, 25) 
            : get().covenantLevel,
          overlayOpacity: settings.overlayOpacity !== undefined 
            ? clamp(settings.overlayOpacity, 0.1, 1) 
            : get().overlayOpacity,
          hasUnsavedChanges: true,
        });
      },

      // Reset to defaults
      resetToDefaults: () => {
        set({ 
          ...DEFAULT_SETTINGS,
          hasUnsavedChanges: true,
        });
      },

      // UI state
      openSettings: () => set({ isSettingsOpen: true }),
      
      closeSettings: () => {
        set({ isSettingsOpen: false });
      },
      
      toggleSettings: () => set({ isSettingsOpen: !get().isSettingsOpen }),
      
      markSaved: () => set({ hasUnsavedChanges: false }),
    }),
    {
      name: 'mt2-draft-settings',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        ocrMode: state.ocrMode,
        covenantLevel: state.covenantLevel,
        overlayOpacity: state.overlayOpacity,
        showCardArt: state.showCardArt,
        enableSoundAlerts: state.enableSoundAlerts,
        theme: state.theme,
      }),
      onRehydrateStorage: () => (state) => {
        if (state) {
          state.hasUnsavedChanges = false;
          state.isSettingsOpen = false;
        }
      },
    }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectOcrSettings = (state: SettingsStore) => ({
  ocrMode: state.ocrMode,
});

export const selectDraftSettings = (state: SettingsStore) => ({
  covenantLevel: state.covenantLevel,
});

export const selectOverlaySettings = (state: SettingsStore) => ({
  overlayOpacity: state.overlayOpacity,
  showCardArt: state.showCardArt,
});

export const selectAllSettings = (state: SettingsStore): AppSettings => ({
  ocrMode: state.ocrMode,
  covenantLevel: state.covenantLevel,
  overlayOpacity: state.overlayOpacity,
  showCardArt: state.showCardArt,
  enableSoundAlerts: state.enableSoundAlerts,
  theme: state.theme,
});
