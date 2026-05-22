import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react';

export type ThemeMode = 'light' | 'dark' | 'system';
type EffectiveTheme = 'light' | 'dark';

type ThemeContextValue = {
  mode: ThemeMode;
  effectiveTheme: EffectiveTheme;
  setMode: (mode: ThemeMode) => void;
};

const storageKey = 'yai-theme';
const modes = new Set<ThemeMode>(['light', 'dark', 'system']);
const ThemeContext = createContext<ThemeContextValue | null>(null);

type ThemeProviderProps = {
  children: ReactNode;
};

function readStoredMode(): ThemeMode {
  if (typeof window === 'undefined') {
    return 'system';
  }

  try {
    const stored = window.localStorage.getItem(storageKey);
    return modes.has(stored as ThemeMode) ? (stored as ThemeMode) : 'system';
  } catch {
    return 'system';
  }
}

function resolveSystemTheme(): EffectiveTheme {
  if (typeof window === 'undefined') {
    return 'light';
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme: EffectiveTheme) {
  document.documentElement.classList.toggle('dark', theme === 'dark');
  document.documentElement.style.colorScheme = theme;
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  const [mode, setModeState] = useState<ThemeMode>(readStoredMode);
  const [systemTheme, setSystemTheme] = useState<EffectiveTheme>(resolveSystemTheme);
  const effectiveTheme = mode === 'system' ? systemTheme : mode;

  useEffect(() => {
    applyTheme(effectiveTheme);
  }, [effectiveTheme]);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = (event: MediaQueryListEvent) => {
      setSystemTheme(event.matches ? 'dark' : 'light');
    };

    setSystemTheme(mediaQuery.matches ? 'dark' : 'light');
    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, []);

  const setMode = useCallback((nextMode: ThemeMode) => {
    setModeState(nextMode);
    try {
      window.localStorage.setItem(storageKey, nextMode);
    } catch {
      // Storage can be unavailable in private browsing or embedded contexts.
    }
  }, []);

  const value = useMemo(
    () => ({
      mode,
      effectiveTheme,
      setMode,
    }),
    [effectiveTheme, mode, setMode],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider');
  }

  return context;
}
