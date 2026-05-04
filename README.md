# Mac Health Monitor Rust

Un tableau de bord macOS natif, léger et local, pour comprendre rapidement la santé d’un Mac sans ouvrir le Terminal ni le Moniteur d’activité.

L’application est écrite en Rust. Elle démarre un petit serveur HTTP local et affiche l’interface dans une vraie fenêtre macOS via `wry` et `tao` avec WKWebView. L’objectif est simple : démarrage rapide, faible consommation, et explications lisibles pour les processus macOS opaques comme `WindowServer`, `fileproviderd`, `cloudd`, `com.apple.WebKit.GPU`, Rosetta ou les outils LLM locaux.

## Fonctionnalités

- Fenêtre macOS native, pas un onglet de navigateur.
- Cartes de suivi CPU, pression mémoire, disque, batterie, réseau, uptime et santé système.
- `Process Watch` sur moyenne glissante de 5 minutes pour CPU, RAM, impact thermique, apps lourdes en veille et suspects Rosetta.
- Explications en langage clair pour les daemons macOS fréquents et les helpers d’apps difficiles à identifier.
- Attribution File Provider au mieux pour iCloud Drive, Adobe Creative Cloud, OneDrive, Dropbox, Google Drive, Box, Nextcloud, Synology Drive et Proton Drive.
- Suivi local de l’activité LLM pour Claude, Codex et Gemini.
- Lecture optionnelle du cache OpenUsage pour afficher quotas, tokens et coûts quand OpenUsage est installé localement.
- Interface claire, chaude et rapide à scanner sur écran de MacBook.

## Confidentialité

L’application tourne localement sur `127.0.0.1` et n’envoie aucune télémétrie vers un serveur distant.

Elle lit des commandes macOS et des fichiers locaux déjà accessibles à l’utilisateur courant. Pour l’usage LLM, elle peut lire le cache OpenUsage :

```text
~/Library/Application Support/com.sunstory.openusage/usage-api-cache.json
```

Elle ne lit pas les conversations Claude, Codex ou Gemini.

## Limites

- Apple Silicon n’expose pas la température CPU précise en degrés Celsius aux apps normales. L’app affiche donc un indice thermique et l’état thermique macOS, sauf si des outils privilégiés exposent plus d’informations.
- Safari et WebKit n’exposent pas d’attribution CPU/RAM fiable par onglet via des API publiques. Les processus WebKit sont expliqués comme activité Safari/webview, mais pas reliés à un onglet exact.
- Les explications de processus sont heuristiques. Elles sont conçues pour être utiles, pas pour faire croire que macOS expose tous les liens privés entre services et apps.
- L’app générée n’est pas notariée. Si elle est distribuée en binaire, macOS peut demander à l’utilisateur de l’autoriser via Gatekeeper.

## Prérequis

- macOS 13 ou plus récent.
- Rust stable avec support de l’édition 2024.
- Xcode Command Line Tools.
- `sips` et `iconutil` pour générer l’icône d’app, disponibles par défaut sur macOS.

## Lancer depuis le code source

```bash
git clone https://github.com/EauZz/mac-health-monitor-rust.git
cd mac-health-monitor-rust
CARGO_TARGET_DIR=/tmp/mac-health-monitor-rust-target cargo run --release
```

Le serveur interne utilise le port `8767` par défaut et bascule automatiquement sur un port libre si nécessaire.

## Construire l’app macOS

```bash
./build-app.sh
open "dist/Mac Health Monitor Rust.app"
```

Le script génère :

```text
dist/Mac Health Monitor Rust.app
```

La sortie peut être personnalisée :

```bash
APP_NAME="Mac Health Monitor" \
BUNDLE_ID="dev.yourname.MacHealthMonitor" \
OUT_DIR="$PWD/dist" \
./build-app.sh
```

## Vérifications de développement

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
node --check public/app.js
./build-app.sh
```

## Structure du repo

```text
src/main.rs          App Rust, serveur local et collecte des métriques macOS
public/             Interface servie dans la fenêtre native
assets/             Ressources de l’icône d’app
build-app.sh        Script portable de génération du bundle .app
PRODUCT.md          Notes produit
DESIGN.md           Direction design
```

## Licence

MIT. Voir [LICENSE](LICENSE).
