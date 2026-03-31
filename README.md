<div align="center">

  <h1>TeleMatch</h1>
  <p><strong>Telegram matchmaking bot postaveny v Ruste</strong></p>
  <p>Objavovanie profilov, swipe flow, mutual matches, viacjazycny onboarding a geolokacne zoradovanie kandidatov v jednej Telegram bot aplikacii.</p>

<p>
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Teloxide-Telegram%20Bot-26A5E4?style=for-the-badge&logo=telegram&logoColor=white" alt="Teloxide Telegram Bot" />
  <img src="https://img.shields.io/badge/PostgreSQL-SQLx-336791?style=for-the-badge&logo=postgresql&logoColor=white" alt="PostgreSQL SQLx" />
  <img src="https://img.shields.io/badge/Google%20Maps-Geocoding-4285F4?style=for-the-badge&logo=googlemaps&logoColor=white" alt="Google Maps Geocoding" />
  <img src="https://img.shields.io/badge/Status-Active%20Development-15803D?style=for-the-badge" alt="Status Active Development" />
</p>

</div>

## Prehlad

TeleMatch je Telegram bot pre zoznamovanie, ktory vedie pouzivatela onboardingom, uklada profil do PostgreSQL, ponuka relevantne profily podla preferencii a vzdialenosti a spravuje lajky aj vzajomne zhody.

Projekt pouziva:

- `teloxide` na Telegram bot logiku a dialogue flow
- `sqlx` na pristup do PostgreSQL a migracie
- `reqwest` na komunikaciu s Google Geocoding API
- `tokio` ako async runtime

## Hlavne funkcie

| Funkcia | Popis |
| --- | --- |
| 🌍 Viacjazycny onboarding | Vyber jazyka a texty pre `English`, `Slovencinu` a `Українську`. |
| 👤 Tvorba profilu | Meno, pohlavie, koho hladas, vek, mesto, bio, fotka, Telegram identita. |
| 📍 Geolokacia | Pouzivatel moze poslat lokalitu ako text alebo priamo zdielat polohu v Telegrame. |
| 🧭 Zoradovanie podla vzdialenosti | Kandidati sa vyberaju podla vzajomnych preferencii a triedia sa podla vzdialenosti medzi mestami. |
| ❤️ Swipe flow | Like / skip rozhodovanie cez Telegram klavesnicu. |
| 🔁 Obnovovanie profilov | Swipe sa po case znovu spristupni; aktualne je nastavene okno `7 dni`. |
| 💘 Mutual match notifikacie | Pri vzajomnom lajku bot posle upozornenie a pri verejnom username aj odkaz na chat. |
| 👀 Fronta prichadzajucich likov | Pouzivatel vie prechadzat ludi, ktorym sa jeho profil pacil. |
| ⚙️ Nastavenia a upravy | Zmena jazyka, uprava profilu, zmena fotky, zmena bio, deaktivacia profilu. |
| 🗃️ Automaticke DB migracie | Migracie sa spustaju automaticky pri starte aplikacie. |

## Ako to funguje

1. Pouzivatel spusti bota cez `/start`.
2. Bot vyziada jazyk a prejde onboarding profilom.
3. Mesto sa prevedie na geograficke suradnice cez Google Geocoding API.
4. Profil sa ulozi do PostgreSQL.
5. Bot ponuka kompatibilne profily a zaznamenava swipe rozhodnutia.
6. Pri vzajomnom lajku odošle match notifikaciu.

## Tech Stack

| Vrstva | Technologie |
| --- | --- |
| Bot runtime | `Rust`, `Tokio`, `Teloxide` |
| Databaza | `PostgreSQL`, `SQLx migrations` |
| Externe API | `Google Maps Geocoding API` |
| Konfiguracia | `.env`, `dotenvy`, `RUST_LOG` |

## Quick Start

### 1. Prerequisites

- Rust toolchain s podporou `edition = "2024"`
- PostgreSQL instancia
- Telegram bot token od `@BotFather`
- Google Maps API key s povolenym Geocoding API

### 2. Konfiguracia prostredia

Vytvor `.env` subor z prikladu:

```powershell
Copy-Item .env-example .env
```

alebo:

```bash
cp .env-example .env
```

Obsah premennych:

| Premenna | Popis |
| --- | --- |
| `TELOXIDE_TOKEN` | Telegram bot token |
| `RUST_LOG` | Uroven logovania, napriklad `info` |
| `DATABASE_URL` | Connection string do PostgreSQL |
| `GOOGLE_MAPS_API_KEY` | API kluc pre geokodovanie miest a polohy |

### 3. Spustenie projektu

```bash
cargo run
```

Pri starte aplikacia:

- nacita `.env`
- inicializuje logger
- pripoji sa na PostgreSQL
- automaticky aplikuje migracie zo `src/db/migrations`
- spusti Telegram dispatcher

## Databaza

Aktualna schema obsahuje tri hlavne oblasti:

- `profiles` pre ulozene pouzivatelske profily, jazyk, chat identitu a suradnice
- `swipes` pre like/skip historiu a cas, kedy sa profil moze objavit znova
- `match_notifications` pre evidenciu uz zobrazenych zhod

## Struktura projektu

```text
src/
|- app/
|  |- handlers/        # onboarding a matching flow
|  |- chat_ui.rs       # Telegram UI helpery a prechody medzi stavmi
|  `- types.rs         # dialog states, language, akcie
|- db/
|  |- migrations/      # SQLx migracie
|  |- profile_repository.rs
|  `- swipe_repository.rs
|- domain/
|  `- profile.rs       # profilovy model a validacia
|- services/
|  `- geocoding.rs     # Google Maps integracia
|- telegram/
|  |- i18n.rs          # lokalizovane texty
|  `- keyboards.rs     # reply keyboards
`- main.rs
```

## Co je dobre vediet

- Dialog stavy su aktualne drzane v `InMemStorage`, teda rozbehnuty flow sa po restarte procesu neobnovi.
- Pri pouzivateloch bez verejneho Telegram username bot nevie vzdy otvorit priamy chat link.

## Vhodne dalsie kroky

- doplnit integracne testy pre onboarding a matching flow
- vyriesit persistentny dialogue storage namiesto `InMemStorage`