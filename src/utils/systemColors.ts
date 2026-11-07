const LIGHT_LUMINANCE_THRESHOLD = 0.55;

const luminance = (hex: string) => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return (0.299 * r + 0.587 * g + 0.114 * b) / 255;
};

/**
 * Applique la couleur de fond (celle de la barre des tâches, ou celle du thème si `null`)
 * et choisit la palette claire ou sombre selon sa luminosité, pour garder le texte lisible.
 */
export const applyAccent = (color: string | null) => {
  const root = document.documentElement;
  if (color) {
    root.style.setProperty("--accent", color);
  } else {
    root.style.removeProperty("--accent");
  }
};

export const applyBackground = (color: string | null) => {
  const root = document.documentElement;

  if (color) {
    root.style.setProperty("--bg", color);
  } else {
    root.style.removeProperty("--bg");
  }

  const isLight = color
    ? luminance(color) > LIGHT_LUMINANCE_THRESHOLD
    : window.matchMedia("(prefers-color-scheme: light)").matches;
  root.dataset.bg = isLight ? "light" : "dark";
};
