# Licences

## Code de Norélias Studio Pro
Licence commerciale **à définir** (`UNLICENSED` dans les manifestes Cargo en attendant). Aucun code tiers copié dans le dépôt :
les dépendances sont tirées de crates.io.

## Dépendances actuelles (lues dans `cargo metadata`, extension incluse)

104 crates externes, **toutes sous licence permissive** compatible avec une distribution commerciale fermée (MIT, Apache-2.0, Zlib, BSD-3-Clause, Unlicense, 0BSD).
Une seule propose une licence double incluant la LGPL (`MIT OR Apache-2.0 OR LGPL-2.1-or-later`) : **on retient MIT**. Les avis de licence doivent être reproduits dans le logiciel distribué.
Nouvelles dépendances de l'extension : `argon2`, `sha2`, `sha1`, `hmac`, `getrandom`, `chacha20poly1305` (RustCrypto), `phonenumber` (Apache-2.0 ; métadonnées dérivées de libphonenumber, Apache-2.0),
`fontdue` (rendu de polices), `png`. **Aucune** dépendance réseau (test automatisé).

| Crate | Version | Licence |
|---|---|---|
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 |
| aead | 0.6.1 | MIT OR Apache-2.0 |
| aho-corasick | 1.1.5 | Unlicense OR MIT |
| allocator-api2 | 0.2.22 | MIT OR Apache-2.0 |
| argon2 | 0.6.0 | MIT OR Apache-2.0 |
| atomic-polyfill | 1.0.3 | MIT OR Apache-2.0 |
| autocfg | 1.5.1 | Apache-2.0 OR MIT |
| base64ct | 1.8.3 | Apache-2.0 OR MIT |
| bitflags | 2.13.2 | MIT OR Apache-2.0 |
| blake2 | 0.11.0 | MIT OR Apache-2.0 |
| block-buffer | 0.12.1 | MIT OR Apache-2.0 |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT |
| byteorder | 1.5.0 | Unlicense OR MIT |
| byteorder-lite | 0.1.0 | Unlicense OR MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 |
| chacha20 | 0.10.2 | MIT OR Apache-2.0 |
| chacha20poly1305 | 0.11.0 | Apache-2.0 OR MIT |
| cipher | 0.5.2 | MIT OR Apache-2.0 |
| cmov | 0.5.4 | Apache-2.0 OR MIT |
| cobs | 0.3.0 | MIT OR Apache-2.0 |
| const-oid | 0.10.3 | Apache-2.0 OR MIT |
| core_maths | 0.1.1 | MIT |
| cpufeatures | 0.3.1 | MIT OR Apache-2.0 |
| crc32fast | 1.5.6 | MIT OR Apache-2.0 |
| critical-section | 1.2.0 | MIT OR Apache-2.0 |
| crypto-common | 0.2.2 | MIT OR Apache-2.0 |
| ctutils | 0.4.8 | Apache-2.0 OR MIT |
| digest | 0.11.3 | MIT OR Apache-2.0 |
| either | 1.19.0 | MIT OR Apache-2.0 |
| embedded-io | 0.4.0 | MIT OR Apache-2.0 |
| embedded-io | 0.6.1 | MIT OR Apache-2.0 |
| equivalent | 1.0.3 | Apache-2.0 OR MIT |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 |
| flate2 | 1.1.11 | MIT OR Apache-2.0 |
| fnv | 1.0.7 | Apache-2.0 / MIT |
| foldhash | 0.1.5 | Zlib |
| fontdue | 0.9.5 | MIT OR Apache-2.0 OR Zlib |
| getrandom | 0.4.3 | MIT OR Apache-2.0 |
| hash32 | 0.2.1 | MIT OR Apache-2.0 |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 |
| heapless | 0.7.17 | MIT OR Apache-2.0 |
| heck | 0.5.0 | MIT OR Apache-2.0 |
| hmac | 0.13.0 | MIT OR Apache-2.0 |
| hybrid-array | 0.4.15 | MIT OR Apache-2.0 |
| image | 0.25.10 | MIT OR Apache-2.0 |
| inout | 0.2.6 | MIT OR Apache-2.0 |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| libc | 0.2.191 | MIT OR Apache-2.0 |
| libm | 0.2.16 | MIT |
| linked-hash-map | 0.5.6 | MIT/Apache-2.0 |
| lock_api | 0.4.14 | MIT OR Apache-2.0 |
| lru-cache | 0.1.2 | MIT/Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| minimal-lexical | 0.2.1 | MIT/Apache-2.0 |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 |
| nom | 7.1.3 | MIT |
| num-traits | 0.2.19 | MIT OR Apache-2.0 |
| once_cell | 1.21.4 | MIT OR Apache-2.0 |
| oncemutex | 0.1.1 | MIT/Apache-2.0 |
| password-hash | 0.6.1 | MIT OR Apache-2.0 |
| phc | 0.6.1 | Apache-2.0 OR MIT |
| phonenumber | 0.3.10+9.0.33 | Apache-2.0 |
| png | 0.18.1 | MIT OR Apache-2.0 |
| poly1305 | 0.9.1 | Apache-2.0 OR MIT |
| postcard | 1.2.0 | MIT OR Apache-2.0 |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 |
| quick-xml | 0.41.0 | MIT |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| r-efi | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| rand_core | 0.10.1 | MIT OR Apache-2.0 |
| regex | 1.13.2 | MIT OR Apache-2.0 |
| regex-automata | 0.4.19 | MIT OR Apache-2.0 |
| regex-cache | 0.2.1 | MIT |
| regex-syntax | 0.6.29 | MIT OR Apache-2.0 |
| regex-syntax | 0.8.12 | MIT OR Apache-2.0 |
| rustc_version | 0.4.1 | MIT OR Apache-2.0 |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 |
| semver | 1.0.28 | MIT OR Apache-2.0 |
| serde | 1.0.229 | MIT OR Apache-2.0 |
| serde_core | 1.0.229 | MIT OR Apache-2.0 |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| sha1 | 0.11.0 | MIT OR Apache-2.0 |
| sha2 | 0.11.2 | MIT OR Apache-2.0 |
| simd-adler32 | 0.3.10 | MIT |
| spin | 0.9.9 | MIT |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 |
| strum | 0.27.2 | MIT |
| strum_macros | 0.27.2 | MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| thiserror | 2.0.21 | MIT OR Apache-2.0 |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 |
| typenum | 1.20.1 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| universal-hash | 0.6.1 | MIT OR Apache-2.0 |
| zlib-rs | 0.6.9 | Zlib |
| zmij | 1.0.23 | MIT |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib |

## Polices embarquées (dans `crates/studio/assets/`, avec leur avis de licence)
| Police | Licence | Usage |
|---|---|---|
| Inter Regular | SIL Open Font License 1.1 (la distribution Debian indique « OFL-1.1 and Apache-2.0 ») | Filigrane « Sans », interface prévue |
| Liberation Serif Italic | SIL Open Font License 1.1 | Filigrane style « Élégant » |

La SIL OFL autorise l'embarquement et la redistribution ; elle interdit de vendre la police **seule** et exige de conserver l'avis de licence et de ne pas utiliser le nom réservé pour une version modifiée.

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
