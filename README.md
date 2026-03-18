# envrac-rust

CLI qui génère et publie automatiquement les articles **"En Vrac"** du blog Hugo de Victor Prouff, à partir des tâches Todoist.

## Fonctionnement

Quand la commande `publish` est lancée, l'outil :

1. Récupère les derniers articles "En Vrac" publiés depuis le dépôt GitHub du blog
2. Récupère les tâches du projet Todoist dédié
3. Groupe les tâches par catégorie (Articles, Youtube, Tools, Podcasts, Livres)
4. Génère le fichier Markdown de l'article avec en-tête et corps
5. Pousse le fichier directement sur le dépôt GitHub via l'API GitHub

## Architecture

```
envrac-rust/
├── run/
│   ├── src/
│   │   ├── main.rs              # CLI (clap), logique principale
│   │   ├── lib.rs               # Exports publics
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── task.rs          # Struct Task (contenu + catégorie)
│   │       ├── category.rs      # Enum Category + mapping section_id → catégorie
│   │       └── github_object.rs # Structs pour l'API GitHub
│   ├── Cargo.toml
│   └── .env                     # Variables d'environnement (non versionné)
└── tests/
    └── envrac_lib_test.rs
```

## Catégories Todoist

Les tâches sont assignées à des sections dans Todoist, chaque section correspondant à une catégorie :

| Catégorie     | Affichage         |
|---------------|-------------------|
| Articles      | 📖 Articles       |
| Youtube       | 🎞️ Youtube       |
| Tools         | 🛠️ Tools         |
| Podcast       | 🎧 Podcasts       |
| Livre         | 📚 Livres         |
| *(autre)*     | ignoré            |

## Prérequis

- [Rust](https://rustup.rs/) (stable)

## Configuration

Créer un fichier `run/.env` :

```env
TODOIST_API_TOKEN=<token Todoist>
GITHUB_API_TOKEN=<token GitHub>
GITHUB_USER_AGENT=<votre username GitHub>
EXECUTOR=<nom de la machine>
```

## Commandes

```bash
# Génère et publie l'article sur le blog
make publish

# Génère l'article et l'affiche dans le terminal (sans publier)
make dry-run

# Build release
make build

# Build l'image Docker
make docker-build
```

Ou directement :

```bash
cd run && set -a && source .env && set +a && cargo run -- publish
cd run && set -a && source .env && set +a && cargo run -- dry-run
```

## Déploiement via CRON (Dokploy)

Le conteneur exécute `run publish` au démarrage. Pour un CRON hebdomadaire, configurer Dokploy pour lancer le conteneur selon le planning souhaité.

## Dépendances principales

- [`clap`](https://github.com/clap-rs/clap) — CLI
- [`reqwest`](https://github.com/seanmonstar/reqwest) — client HTTP (Todoist, GitHub)
- [`serde`](https://serde.rs/) — sérialisation/désérialisation JSON
- [`chrono`](https://github.com/chronotope/chrono) — formatage des dates
- [`base64`](https://github.com/marshallpierce/rust-base64) — encodage du contenu pour l'API GitHub
