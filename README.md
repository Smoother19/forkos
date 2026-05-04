# forkOS

Un fork d'Arch Linux centré sur deux idées :

1. **Architecture immuable** — la partition OS est en lecture seule, les
   mises à jour sont atomiques (système A/B), les données utilisateur
   sont strictement séparées et persistantes. Les apps tournent isolées
   sans sacrifier les performances.

2. **Interface graphique narrative** — fusion entre CLI et GUI, sans
   fenêtres flottantes ni onglets. Un fil narratif comme bureau d'accueil,
   une seule app plein écran à la fois, une palette `⌘K` comme invocation
   universelle.

## État du projet

🚧 En développement actif — phase de prototypage.

## Structure du repo

- `crates/` — composants Rust (palette, menu narratif, libs partagées)
- `os/` — infrastructure OS (hooks, systemd, build ISO)
- `config/` — configs des composants tiers (niri, thèmes)
- `docs/` — documentation technique et décisions de design
- `scripts/` — scripts de développement et déploiement

## Stack technique

- **Base** : Arch Linux + Btrfs + systemd-boot (A/B)
- **Apps isolées** : Flatpak (GUI), Distrobox (dev), pacman (système)
- **Compositor** : niri (fork ou config custom)
- **Shell graphique** : Rust + iced + smithay-client-toolkit
- **Recherche** : nucleo (fuzzy matching)
