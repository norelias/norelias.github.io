# Plan de tests

Pyramide : unitaires → scénarios sur images → bout en bout (binaire réel) → visuels/matériels (prévus).
Lancer : `cargo test --workspace` (≈ 15 s) ; mesures : `cargo test --release -p norelias-advisor --test perf -- --ignored --nocapture`.

## 1. Résultats actuels (Linux x86-64, Rust 1.97, 4 cœurs)

111 tests, 0 échec ; 1 test de performance (ignoré par défaut).

| Suite | Tests | Ce qui est vérifié |
|---|---|---|
| `norelias-core` | 16 | Aller-retour sRGB exact sur 256 valeurs ; chaque opération (identité, valeurs connues) ; source jamais modifiée ; sérialisation ; débruitage préserve les contours |
| `norelias-advisor` (scénarios) | 13 | 9 défauts injectés (sous/sur-exposition + écrêtage, dominante chaude, teinte verte, bruit, flou, bougé, horizon incliné −4°/2°/5°, sujet centré) détectés **puis corrigés** et ré-analysés ; aucun faux positif sur la scène de référence ; aucune correction inventée pour le bougé ; corrections créatives jamais automatiques ; textes FR/EN |
| `norelias-onboarding` | 48 | Profil, migrations, corruption, version future, écriture atomique, effacement ; consentements ; registre d'outils ; **aucun outil supprimé** (3 modes × 14 spécialités × aides on/off) ; raccourcis sans conflit ; import XMP ; questionnaire (réponses complètes, passer, entrées invalides, fermeture, révocation) |
| `norelias-academy` | 26 | Leçons FR/EN complètes et liées aux outils réels ; exercices (inactivité = échec, correction du conseiller = réussite, mauvaises réponses = échec) ; recherche ; infobulles ; suivi facultatif ; aide contextuelle ; aides désactivées |
| `norelias-cli` (e2e) | 8 | Binaire réel sur vrais PNG : analyse, correction automatique/choisie, **original intact**, refus d'écraser, questionnaire par stdin, profil local, workspaces, recherche, garde anti-réseau |

Mesures réelles (24 Mpx, mono-thread) : voir ARCHITECTURE.md §5.

## 2. Limites des tests actuels (à lire avant de se fier aux résultats)

* **Données synthétiques uniquement.** Les seuils du conseiller ne sont pas calibrés sur de vraies photographies ; faux positifs
  attendus (contre-jour voulu, scènes sans contours, horizons naturellement inclinés). Dit à l'utilisateur dans chaque explication.
* **Linux uniquement.** Aucun test Windows/macOS.
* **Aucune interface graphique** donc aucun test visuel ni d'accessibilité.
* L'e2e utilise un dossier de données temporaire ; aucun test d'installation.

## 3. Tests obligatoires du brief — statut

| Test requis | Statut |
|---|---|
| Importation et développement de vrais RAW | prévu (aucun décodeur) |
| Import Nikon D5200 NEF | prévu — **fichier NEF autorisé à fournir** |
| Fidélité des couleurs (ICC, ΔE) | prévu (sRGB aller-retour exact testé seulement) |
| Précision des masques | prévu |
| Préservation des originaux | **testé** (moteur + CLI) |
| Réouverture des projets et calques | prévu |
| Annulation / rétablissement | prévu (les `Op` le permettront) |
| Cohérence aperçu / export | partiel : l'aperçu et l'application utilisent les mêmes `Op` ; test d'égalité numérique à ajouter |
| Centaines de photos / catalogues volumineux | prévu |
| Mémoire GPU insuffisante | prévu |
| Hors connexion | **testé** (aucune dépendance réseau) |
| Récupération après plantage | partiel (profil : fichier corrompu/tronqué) ; projets : prévu |
| Impression et profils ICC | prévu |
| Windows et macOS | prévu (CI à créer) |
| Confidentialité des photographies | **testé** (garde réseau, consentements, aucune écriture en dehors des sorties demandées) |

## 4. Plan du module M (couverture des exigences)

| Exigence | Tests |
|---|---|
| M-01…M-03 | `questionnaire::tests::*`, e2e `questionnaire_over_stdin…`, `store::tests::*` |
| M-04…M-08 | `workspace::tests::*`, `profile::tests::mode_suggestions` |
| M-09…M-12 | `advisor/tests/scenarios.rs` (13) |
| M-13…M-22 | `academy::*::tests` (26), e2e `academy_search…` |
| M-23…M-26 | `shortcuts::tests::*`, `compat::tests::*` |
| M-27…M-31 | `privacy::tests::*`, e2e `first_run…`, `no_networking…` |

## 5. À ajouter

Jeu de photographies autorisées (RAW + JPEG) avec vérités terrain pour calibrer le conseiller ; tests visuels de non-régression sur
images de référence ; CI Windows/macOS/Linux ; fuzzing des décodeurs ; tests utilisateurs de l'onboarding (taux de complétion,
compréhension des modes, profils de raccourcis).
