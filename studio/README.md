# Norélias Studio Pro

Application de photographie professionnelle (Windows et macOS) — **projet en phase 1, état v0.1**.

> **État honnête.** Ce dépôt contient aujourd'hui une *fondation réelle et testée*, pas un logiciel de
> retouche complet. Il n'y a **pas encore** d'interface graphique de bureau, de décodage RAW, de catalogue,
> de GPU, de calques, d'IA ni d'impression. Chaque fonction listée comme « testée » dans
> [FEATURE_MATRIX.md](FEATURE_MATRIX.md) a un moteur réel et des tests qui passent ; tout le reste est
> « prévu ». Aucun bouton factice : un outil non intégré est marqué `Planned` dans le registre et n'est
> jamais présenté comme fonctionnel.

## Ce qui existe (v0.1)

| Crate | Rôle |
|---|---|
| `norelias-core` | Moteur d'image **indépendant de toute interface** : pixels RVB linéaires `f32`, 10 opérations non destructives sérialisables (exposition, contraste, balance des blancs, hautes lumières, ombres, saturation, réduction du bruit, netteté, rotation, recadrage), images d'entraînement procédurales |
| `norelias-advisor` | Conseiller photographique : mesure réellement les pixels (exposition, balance des blancs, teinte, écrêtage, bruit, netteté, flou de mouvement, horizon, composition), explique en FR/EN, propose des corrections prévisualisables |
| `norelias-onboarding` | Questionnaire initial, profil local versionné, trois modes d'espace de travail + espaces par spécialité, registre d'outils, profils de raccourcis, import de presets XMP, consentements |
| `norelias-academy` | 20 leçons FR/EN, 7 exercices vérifiés par le vrai conseiller, recherche, infobulles, aide contextuelle, suivi facultatif |
| `norelias-cli` | Binaire `norelias` : preuve de bout en bout sur de vrais fichiers PNG/JPEG |

## Construire et tester

```bash
cd studio
cargo test --workspace          # 111 tests : unitaires, scénarios sur images, bout en bout (binaire réel)
cargo build --release
./target/release/norelias academy training ex_exposure sombre.png       # photo d'entraînement libre de droits
./target/release/norelias analyze sombre.png --lang fr                  # diagnostic expliqué
./target/release/norelias analyze sombre.png --auto --out corrigee.png  # l'original n'est jamais modifié
./target/release/norelias profile init                                  # questionnaire (s = passer)
./target/release/norelias workspace beginner portrait                   # aucun outil supprimé, jamais
```

Prérequis : Rust ≥ 1.80 (testé avec 1.97). Aucune dépendance native, aucune bibliothèque réseau
(un test échoue si l'une d'elles entre dans `Cargo.lock`).

## Documents

[SPECIFICATION](SPECIFICATION.md) · [ARCHITECTURE](ARCHITECTURE.md) · [DESIGN_SYSTEM](DESIGN_SYSTEM.md) ·
[FEATURE_MATRIX](FEATURE_MATRIX.md) · [ROADMAP](ROADMAP.md) · [TEST_PLAN](TEST_PLAN.md) ·
[LICENSES](LICENSES.md) · [SECURITY](SECURITY.md) · [docs/MIGRATION.md](docs/MIGRATION.md)

## Attention : publication

Ce dossier vit dans le dépôt du site `norelias.github.io`. Si cette branche est fusionnée dans la branche
publiée par GitHub Pages, **le code source et les documents seraient servis publiquement** sous
`norelias.ca/studio/`. Pour un logiciel commercial, il est préférable de déplacer `studio/` dans un dépôt privé.
