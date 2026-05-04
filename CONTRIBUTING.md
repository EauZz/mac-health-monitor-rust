# Contribuer

Les contributions sont bienvenues si elles respectent l’objectif du projet : une app légère, locale-first et compréhensible par des utilisateurs Mac non experts.

## Installation locale

```bash
git clone https://github.com/EauZz/mac-health-monitor-rust.git
cd mac-health-monitor-rust
CARGO_TARGET_DIR=/tmp/mac-health-monitor-rust-target cargo run --release
```

## Avant d’ouvrir une pull request

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
node --check public/app.js
./build-app.sh
```

## Principes de contribution

- Privilégier les API publiques macOS et les permissions utilisateur normales.
- Ne pas ajouter de télémétrie ni d’analyse distante.
- Ne pas lire les transcripts LLM ni les fichiers privés de conversation.
- Garder les explications de processus honnêtes quand l’attribution est heuristique.
- Éviter les dépendances front-end lourdes sauf raison technique solide.

## Bonnes premières contributions

- Ajouter des explications pour davantage de daemons macOS.
- Améliorer l’attribution File Provider.
- Améliorer l’accessibilité et le comportement responsive.
- Ajouter des captures d’écran et améliorer le packaging de publication.
