# envrac-rust

API HTTP qui génère et publie automatiquement les articles **"En Vrac"** du blog Hugo de Victor Prouff, à partir des tâches Todoist.

## Fonctionnement

Quand l'endpoint `POST /en-vrac` est appelé (avec le bon secret), l'API :

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
│   │   ├── main.rs              # Serveur warp, routes, logique principale
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
SECRET=<secret pour protéger l'endpoint>
```

## Lancer en local

```bash
cd run && set -a && source .env && set +a && cargo run
```

Le serveur démarre sur le port **3030**.

## Endpoints

| Méthode | Route          | Description                          |
|---------|----------------|--------------------------------------|
| `POST`  | `/en-vrac?secret=<SECRET>` | Génère et publie l'article |
| `GET`   | `/healthcheck` | Vérifie que le serveur est actif      |

## Dépendances principales

- [`warp`](https://github.com/seanmonstar/warp) — serveur HTTP
- [`reqwest`](https://github.com/seanmonstar/reqwest) — client HTTP (Todoist, GitHub)
- [`serde`](https://serde.rs/) — sérialisation/désérialisation JSON
- [`chrono`](https://github.com/chronotope/chrono) — formatage des dates
- [`base64`](https://github.com/marshallpierce/rust-base64) — encodage du contenu pour l'API GitHub
