# Architecture forkOS

## Vision

Système immuable basé sur Arch, avec séparation stricte OS/données et
isolation des applications. Interface graphique radicalement différente
des paradigmes classiques.

## Principes techniques

### Immutabilité

- `/usr` monté en lecture seule
- Système A/B : deux subvolumes Btrfs root, swap au reboot
- Snapshots automatiques avant chaque mise à jour (hook pacman)
- Rollback en un seul reboot

### Séparation OS / données

- `/usr` : OS immuable, change atomiquement
- `/home` : données utilisateur, jamais touchées par les MAJ
- `/var` : état système persistant
- `/apps` ou `/var/lib/flatpak` : apps isolées

### Isolation des applications

| Type | Outil | Cas d'usage |
|------|-------|-------------|
| GUI utilisateur | Flatpak | Apps quotidiennes (browser, lecteur, etc.) |
| Dev / CLI | Distrobox | Environnements de développement |
| Système | pacman | Composants critiques de base |

## Stack technique détaillée

À compléter au fur et à mesure des décisions.
