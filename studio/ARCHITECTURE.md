# Architecture

## 1. Principes

1. **Le moteur ne connaît pas l'interface.** `norelias-core` (pixels et opérations) n'a aucune dépendance graphique ni réseau.
2. **Les réglages sont des données.** Une `Op` est sérialisable (serde) ; l'image source n'est jamais modifiée.
   Presets, historique, copies virtuelles et projets seront des listes d'`Op`.
3. **Un registre unique d'outils** (`norelias-onboarding::tools`) alimente espaces de travail, infobulles, recherche et matrice.
   Un test garantit que chaque opération du moteur y figure et que tout outil « disponible » a un vrai moteur.
4. **Confidentialité par construction.** Aucune bibliothèque réseau dans le graphe de dépendances (test automatisé) ;
   toute sortie de données doit passer par `ConsentLedger::require`.
5. **Honnêteté du statut.** `Status::Planned` interdit l'affichage actif d'un outil non intégré.

## 2. Vue des crates (état réel v0.1)

```
norelias-core        pixels f32 linéaires · Op (10) · couleur sRGB · images synthétiques
   ▲
norelias-advisor     mesures + constats + corrections prévisualisables
   ▲                          ▲
norelias-onboarding ◄─────────┘ (core seulement)   profil · store · privacy · tools · workspace · shortcuts · compat · questionnaire
   ▲
norelias-academy     leçons · exercices · recherche · infobulles · progression · aide contextuelle
   ▲
norelias-cli         binaire `norelias` (E/S d'images via la crate `image`)
```

## 3. Registre des décisions (ADR)

| ADR | Décision | Justification | Statut |
|---|---|---|---|
| 001 | Moteur et logique en **Rust** | Sûreté mémoire, performances, un seul code pour Windows/macOS, tests natifs | Adoptée |
| 002 | Pixels **RVB linéaires `f32`** | Les opérations d'exposition/balance des blancs sont physiquement correctes en linéaire ; marge >1,0 pour récupérer les hautes lumières | Adoptée |
| 003 | Conversion sRGB uniquement à l'entrée/sortie ; profils ICC (Little CMS) à intégrer | Conversion explicite exigée par le cahier des charges | sRGB adoptée ; ICC **prévu** |
| 004 | Coque desktop **Tauri** + interface React/TypeScript | Binaire léger, WebView native, backend Rust | **À valider par prototype** : non installé ni testé ici |
| 005 | Profil utilisateur = JSON local versionné, écriture atomique (tmp → fsync → rename), `.bak`, migrations | Pas de compte, lisible, récupérable | Adoptée, testée |
| 006 | Conseiller = **heuristiques de traitement du signal**, pas d'apprentissage | Explicables, hors ligne, sans modèle à distribuer ; ML ajouté plus tard pour les cas où elles échouent | Adoptée |
| 007 | Images d'entraînement **procédurales** | Droits garantis, déterministes, testables | Adoptée |
| 008 | Aucune dépendance réseau dans les crates locales | Confidentialité ; test `no_networking_crate_in_dependency_graph` | Adoptée |
| 009 | Raccourcis « familiers » par conventions d'usage, sans ressource propriétaire | Légalité et identité propre | Adoptée |
| 010 | GPU : `wgpu` (Vulkan/Metal/DX12) avec repli CPU | Un seul backend portable, MIT/Apache-2.0 | **Prévue**, à prototyper |
| 011 | RAW : LibRaw lié dynamiquement (LGPL/CDDL) | Maturité, couverture des appareils (dont Nikon NEF) | **Prévue**, licence à valider |
| 012 | Catalogue : SQLite avec migrations versionnées | Domaine public, fiable | **Prévue** |

## 4. Bibliothèques envisagées — évaluation (licences à **re-vérifier** avant toute intégration)

Les licences ci-dessous viennent de la connaissance générale, **non vérifiée dans cet environnement** ; seules les
dépendances réellement présentes (LICENSES.md) ont été lues dans les métadonnées Cargo.

| Bibliothèque | Usage | Licence (à vérifier) | Point d'attention commercial |
|---|---|---|---|
| Tauri | Coque desktop | MIT / Apache-2.0 | Dépend de WebView2 (Windows) et WKWebView (macOS) |
| wgpu | GPU | MIT / Apache-2.0 | Pilotes variables ; repli CPU obligatoire |
| LibRaw | RAW | LGPL-2.1 ou CDDL-1.0 | Liaison dynamique, fournir les sources de la bibliothèque |
| Little CMS | ICC | MIT | — |
| OpenImageIO | Formats | Apache-2.0 (versions récentes) | Poids de la dépendance |
| OpenCV | Vision | Apache-2.0 (≥ 4.5) | Modules contrib non libres à éviter |
| ONNX Runtime | IA locale | MIT | Modèles : **chaque modèle a sa propre licence** |
| SQLite | Catalogue | Domaine public | — |
| Inter (police) | Interface | SIL OFL 1.1 | Embarquable librement |

## 5. Performances mesurées (v0.1, CPU mono-thread, 24 Mpx, `--release`)

| Opération | Durée |
|---|---|
| Analyse complète du conseiller | 0,19 s |
| Aperçu d'une correction à 1024 px | 0,04 s |
| Exposition + balance des blancs + hautes lumières (pleine résolution) | 0,95 s |
| Netteté (pleine résolution) | 1,3 s |
| Réduction du bruit (pleine résolution) | **4,2 s** |
| Mémoire d'une image 24 Mpx en `f32` RVB | 288 Mo |

Ces chiffres sont **insuffisants** pour des curseurs fluides en pleine résolution : le moteur traite l'image entière,
sans tuiles, SIMD, multithreading ni GPU. Voir ROADMAP (phase 1 → 2) : rendu par tuiles, pyramide d'aperçus, cache,
annulation, puis GPU. Machines de référence prévues : 32 Go + RTX 3060 8 Go, et sans GPU dédié — **non mesurées**
(les mesures ci-dessus viennent d'un conteneur Linux de 4 cœurs).

## 6. Limites connues du moteur v0

* Pas de tuiles, pas de GPU, pas de SIMD ; traitement image entière.
* Pas de gestion ICC ni de gamut ; entrée 8 bits sRGB.
* Réduction du bruit et netteté en 3×3 : qualité de démonstration, pas de niveau professionnel.
* Le conseiller est calibré sur des scènes synthétiques uniquement.
