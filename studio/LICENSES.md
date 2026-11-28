# Licences

## Code de Norélias Studio Pro
Licence commerciale **à définir** (`UNLICENSED` dans les manifestes Cargo en attendant). Aucun code tiers copié dans le dépôt :
les dépendances sont tirées de crates.io.

## Dépendances actuelles (lues dans `cargo metadata`, v0.1)

Toutes sont sous licence permissive compatible avec une distribution commerciale fermée (MIT, Apache-2.0, Zlib, BSD-3-Clause,
Unlicense, 0BSD). Il faut tout de même reproduire leurs avis de licence dans le logiciel distribué. `serde` et `serde_json` sont
utilisées par le moteur ; `image` et ses dépendances **uniquement par la CLI** (lecture/écriture PNG/JPEG).

| Crate | Version | Licence |
|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| byteorder-lite | 0.1.0 | Unlicense OR MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| crc32fast | 1.5.6 | MIT OR Apache-2.0 |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| flate2 | 1.1.11 | MIT OR Apache-2.0 |
| image | 0.25.10 | MIT OR Apache-2.0 |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| png | 0.18.1 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.10 | MIT |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| zlib-rs | 0.6.9 | Zlib |
| zmij | 1.0.23 | MIT |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib |

Vérifier à chaque ajout de dépendance : `cargo metadata --format-version 1` puis lecture du champ `license` ; refuser GPL/AGPL
et toute licence « non commerciale ». Un outil d'audit automatisé (`cargo-deny`) est à ajouter en CI.

## Contenus
* **Photographies d'entraînement** : générées par le code (`norelias_core::synth`), aucun droit tiers.
* **Textes de l'Académie** : rédigés pour ce projet ; à faire relire.
* **Police Inter** (prévue) : SIL OFL 1.1, embarquable.
* **Aucune** icône, disposition, texte ou préréglage d'un éditeur tiers (Adobe, Capture One, Skylum, Topaz, Evoto, Camera Bits…) n'est utilisé.
  Les noms de logiciels n'apparaissent que pour décrire la compatibilité (raccourcis, XMP) ; ce sont des marques de leurs propriétaires,
  sans affiliation ni approbation.

## Bibliothèques prévues
Voir ARCHITECTURE.md §4 : licences **non vérifiées** à ce jour, à contrôler avant intégration (LibRaw : LGPL/CDDL — attention à la liaison).
Modèles IA : chaque modèle a sa propre licence (usage commercial, redistribution) ; le gestionnaire de modèles devra l'afficher et la vérifier.
