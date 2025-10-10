/**
 * Convert bytes into human readable format
 * @param bytes    - Number of bytes
 * @param decimals - How many decimals to keep
 * @returns          The human readable format (example : 1.5 MB)
 */
export function calculateHumanReadableFileSize(
  bytes: number,
  decimals = 1,
): string {
  if (bytes === 0) return "0 B";

  const k = 1000;
  const sizes = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const size = bytes / Math.pow(k, i);

  return `${size.toFixed(decimals)} ${sizes[i]}`;
}

/**
 * Apply theme if one is already saved in localStorage,
 * else set it to default theme and colorscheme.
 */
export function applyTheme(): void {
  // get persisted theme & colorscheme preferrences, if they are saved
  // else set theme and colorscheme to default values
  const theme = localStorage.getItem("theme") ?? "dark";
  const colorScheme = localStorage.getItem("colorscheme") ?? "gruvbox";

  /* if theme is set to system prefered
   then set the colorscheme based on prefers-color-scheme */
  if (theme === "system") {
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    document.documentElement.setAttribute(
      "data-theme",
      isDark ? `${colorScheme}-dark` : `${colorScheme}-light`,
    );
  } else {
    document.documentElement.setAttribute(
      "data-theme",
      `${colorScheme}-${theme}`,
    );
  }
}

export type Theme = "light" | "dark";
export type ColorScheme = "gruvbox" | "solarized";
export type SfxEnabled = "true" | "false";

interface LocalStorageDefaults {
  theme: Theme;
  colorscheme: ColorScheme;
  isSfxEnabled: SfxEnabled;
}

export const VALID_THEMES: Theme[] = ["light", "dark"];
export const VALID_COLORSCHEMES: ColorScheme[] = ["gruvbox", "solarized"];
export const VALID_SFX: SfxEnabled[] = ["true", "false"];

/**
 * Validate all the data stored in localstorage, if the values are not valid, set them to default values
 * @param key          - The key of value stored in localstorage
 * @param validValues  - The list of valid values for the given key
 * @param defaultValue - The default value for the given key
 * @returns              void
 */
export function validateLocalStorageItem<K extends keyof LocalStorageDefaults>(
  key: K,
  validValues: LocalStorageDefaults[K][],
  defaultValue: LocalStorageDefaults[K],
): void {
  const value = localStorage.getItem(key) as LocalStorageDefaults[K] | null;
  if (!value || !validValues.includes(value)) {
    localStorage.setItem(key, defaultValue);
    console.warn(`Invalid ${key} value. Reset to "${defaultValue}".`);
  }
}
