# Bomberman

Un Bomberman en 3D écrit en Rust avec Vulkan, pour le cursus de 42 Paris. C'est un jeu
complet : une campagne de 5 niveaux avec une histoire, un niveau par monde et le boss final
dans le dernier, une arène multijoueur locale jusqu'à 8 joueurs ou IA, la prise en charge des
manettes et des sauvegardes.

## Démarrage rapide

```bash
./scripts/setup.sh
./Bomberman
```

ou, si Rust est déjà installé :

```bash
make
make run
```

## Assets

Les assets ne sont pas dans le dépôt : `make` télécharge `assets.zip` depuis la release
`assets-v1` et vérifie son empreinte (`scripts/assets.sha256`). `make pack-assets` régénère
ce zip à partir du dossier `assets/` local.
