# Spotify Widget

Petit widget Windows (Tauri 2 + React + TypeScript) qui reste au-dessus de la barre des tâches et affiche la musique en cours de lecture (via SMTC, la couche média de Windows).

## Prérequis

- Windows 10 ou 11
- [Node.js](https://nodejs.org/) et npm
- [Rust](https://rustup.rs/) (via `rustup`) avec les Build Tools Visual Studio (charge de travail « Développement Desktop en C++ »)
- WebView2 (déjà présent avec Edge sur la plupart des machines)

## Installation

```bash
npm install
```

## Lancer en développement

```bash
npm run tauri dev
```

La première compilation Rust est longue (plusieurs minutes). Un changement de `src-tauri/tauri.conf.json` ou du code Rust demande un redémarrage ou déclenche une recompilation ; le front (React) se recharge à chaud.

## Commandes utiles

| Commande | Rôle |
|---|---|
| `npm run tauri dev` | Lance l'app en mode développement |
| `npm run tauri build` | Produit l'exécutable et l'installateur (`src-tauri/target/release`) |
| `npm run dev` | Lance seulement le front dans le navigateur (sans Tauri) |
| `npm run build` | Vérifie les types et construit le front |

Côté Rust, depuis `src-tauri/` :

| Commande | Rôle |
|---|---|
| `cargo check` | Compile sans produire de binaire (rapide) |
| `cargo clippy` | Linter Rust |
| `cargo fmt` | Formate le code |

## Structure

```
src/          Front React / TypeScript
src-tauri/    Backend Rust et configuration Tauri
  src/lib.rs  Point d'entrée de l'app (fonction run)
  tauri.conf.json  Configuration de la fenêtre et du build
```

## Environnement de développement conseillé

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
