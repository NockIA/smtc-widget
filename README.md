# Spotify Widget

Petit widget Windows (Tauri 2 + React + TypeScript) qui reste au-dessus de la barre des tâches et affiche la musique / media en cours de lecture via SMTC.

## Prérequis

- Windows 10 ou 11 (actuellement setup pour Windows 10)
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

| Commande              | Rôle                                                                |
| --------------------- | ------------------------------------------------------------------- |
| `npm run tauri dev`   | Lance l'app en mode développement                                   |
| `npm run tauri build` | Produit l'exécutable et l'installateur (`src-tauri/target/release`) |
| `npm run dev`         | Lance seulement le front dans le navigateur (sans Tauri)            |
| `npm run build`       | Vérifie les types et construit le front                             |

Côté Rust, depuis `src-tauri/` :

| Commande       | Rôle                                      |
| -------------- | ----------------------------------------- |
| `cargo check`  | Compile sans produire de binaire (rapide) |
| `cargo clippy` | Linter Rust                               |
| `cargo fmt`    | Formate le code                           |

## Structure

Le code est organisé **par feature** : chaque dossier de `features/` regroupe ses modèles,
sa logique et ses commandes Tauri.

```
src/                          Front React / TypeScript
  App.tsx                     Écoute des événements du backend (morceau, couleurs, visibilité)
  components/
    IconButton/               Bouton rond avec icône SVG
    SpotifyWidget/            Widget (titre défilant, pochette, progression, volume, contrôles, poignée)
  hooks/                      useAlbumArt, useHoverPanel, useSystemColors, useVolume
  styles/                     Variables de couleur (_colors.css), reset, styles globaux
  types/                      Types partagés avec le backend (morceau, couleurs système)
  utils/                      Application des couleurs système (fond, accent)

src-tauri/                    Backend Rust et configuration Tauri
  tauri.conf.json             Configuration de la fenêtre et du build
  capabilities/               Permissions Tauri
  src/
    lib.rs                    Point d'entrée : plugins, setup, liste des commandes
    shared/                   Code commun : erreur de commande (AppError), manager SMTC partagé
    features/
      playback/               Morceau en cours, contrôles, pochette, écoute des événements SMTC
      audio_source/           Source audio choisie (état, liste des sources)
      volume/                 Volume par application (WASAPI)
      window/                 Placement par écran, position sauvegardée, premier plan, écrans branchés
      preferences/            Fichier de préférences unique (thème, position par écran)
      appearance/             Thème appliqué, couleurs système (barre des tâches, accent)
      context_menu/           Menu du clic droit (piste audio, écran, thème, quitter)
```

Dans chaque feature, les fichiers sont répartis par rôle : `models.rs` (types),
`service.rs` ou `services/` (logique), `commands.rs` (commandes Tauri), `state.rs` (état partagé),
`constants.rs` (constantes).

## Environnement de développement conseillé

- [Zed](https://zed.dev/) avec l'extension Rust (rust-analyzer est intégré)

