# Design de l'interface graphique

## Concept central

Trois primitives, c'est tout :

1. **Menu narratif** — bureau d'accueil, fil de session chaleureux,
   tout est cliquable. État par défaut.
2. **App plein écran** — focus total sur une seule tâche, pas de chrome.
3. **Palette ⌘K** — invocation universelle, fuzzy matching, modes
   contextuels.

## Navigation

- État par défaut : menu narratif
- Clic / commande → app plein écran
- `esc` → retour menu
- `⌘K` → palette en overlay
- `⌘tab` → cycle entre apps actives (raccourci pour ping-pong)

## Esthétique

Palette de couleurs Rose Pine Dawn (clair, chaud, lisible).
Typographie : monospace pour les éléments structurels, sans-serif pour
le contenu d'app.
