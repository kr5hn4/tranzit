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

export function applyTheme() {
  // get persisted theme & colorscheme preferrences
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
