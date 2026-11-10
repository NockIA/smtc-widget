const LIGHT_LUMINANCE_THRESHOLD = 0.55;

const luminance = (hex: string) => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return (0.299 * r + 0.587 * g + 0.114 * b) / 255;
};

const setOrRemoveProperty = (name: string, value: string | null) => {
  const { style } = document.documentElement;
  if (value) {
    style.setProperty(name, value);
  } else {
    style.removeProperty(name);
  }
};

export const applyAccent = (color: string | null) => setOrRemoveProperty("--accent", color);

/**
 * Applique la couleur de fond (celle de la barre des tâches, ou celle du thème si `null`)
 * et choisit la palette claire ou sombre selon sa luminosité, pour garder le texte lisible.
 * Avec `transparent`, le fond n'est pas peint mais la palette reste choisie d'après cette couleur.
 */
export const applyBackground = (color: string | null, transparent = false) => {
  setOrRemoveProperty("--bg", transparent ? "transparent" : color);

  const isLight = color
    ? luminance(color) > LIGHT_LUMINANCE_THRESHOLD
    : window.matchMedia("(prefers-color-scheme: light)").matches;
  document.documentElement.dataset.bg = isLight ? "light" : "dark";
};
