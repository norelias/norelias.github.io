# Matrice des fonctionnalités et registre des exigences

Statuts : **prévu** · **en développement** · **intégré** (moteur réel dans le code) · **testé** (tests automatisés passent) · **validé** (vérifié sur vraies photos / vrai matériel / vrais utilisateurs).

> **Aucune ligne n'est « validée »** : aucune validation sur fichiers RAW réels, matériel ou utilisateurs n'a eu lieu.
> « Testé » signifie : tests automatisés sur images **synthétiques** générées par le code, sur Linux uniquement.
> Les preuves renvoient à des fichiers du dépôt. 111 tests passent (`cargo test --workspace`).

## Exigences non négociables

| ID | Exigence | Statut | Preuve / remarque |
|---|---|---|---|
| N-01 | Application desktop réelle | prévu | Moteur et CLI uniquement ; l'interface Tauri n'est pas commencée |
| N-02 | Fonctionnement local sans Internet | testé | Aucune dépendance réseau (`no_networking_crate_in_dependency_graph`) |
| N-03 | Interface FR/EN | en développement | Textes du conseiller, de l'Académie, du questionnaire et des infobulles en FR/EN testés ; pas d'interface graphique |
| N-04 | Originaux jamais écrasés | testé | CLI : `never_overwrites_original_or_existing_file` ; le moteur ne modifie jamais sa source (`source_is_never_modified`) |
| N-05 | Retouche non destructive | testé (moteur) | `Op` sérialisable, `apply_stack` ; format de projet : prévu |
| N-06 | Couleur haute précision | en développement | RVB linéaire `f32`, sRGB↔linéaire exact aux 256 valeurs ; **pas de gestion ICC** (sRGB supposé) |
| N-07 | GPU + repli CPU | prévu | CPU mono-thread uniquement |
| N-08 | RAW haute résolution, gros catalogues | prévu | Aucun décodeur RAW, aucun catalogue |
| N-09 | Écrans haute résolution | prévu | Dépend de l'interface |
| N-10 | Sauvegarde auto et récupération | en développement | Profil : écriture atomique, `.bak`, quarantaine du fichier corrompu (testé) ; projets/catalogue : prévu |
| N-11 | Aucun bouton factice | testé (registre) | Outils `Planned` jamais décrits comme fonctionnels (`planned_tools_never_get_a_functional_description`) |
| N-12 | Pas de dépendance commerciale imposée | testé | 31 dépendances, toutes permissives (LICENSES.md) |
| N-13 | Modularité, moteur indépendant de l'UI | testé | `norelias-core` sans UI ; 5 crates |
| N-14 | Confidentialité des photos clients | testé (partiel) | Aucun code réseau ; consentements par finalité ; hébergement de galeries : prévu |

## Module M — Personnalisation et apprentissage

| ID | Fonction | Statut | Preuve / remarque |
|---|---|---|---|
| M-01a | Questionnaire : les 11 sujets (logique, texte FR/EN) | testé | `questionnaire.rs` (8 tests) + `questionnaire_over_stdin_persists_answers…` |
| M-01b | Questionnaire : interface graphique | prévu | |
| M-02 | Passer le questionnaire, ne jamais le redemander | testé | `skip_at_any_point…`, `skipping_never_asks_again_but_can_reopen` |
| M-03 | Modifier toutes les réponses plus tard | testé (données) | `answers_can_be_changed_later`, `reopen()` ; écran des préférences : prévu |
| M-04 | Trois modes, même moteur | testé | `same_engine_tools_usable_in_every_mode` |
| M-05 | Visibilité des panneaux, détail des réglages, explications, raccourcis, recommandations, tutoriels par mode | testé (modèle) | `Layout::build` ; rendu dans l'interface : prévu |
| M-06 | Jamais de suppression d'outil selon le niveau | testé | `no_mode_or_specialty_ever_removes_a_tool` (3 modes × 14 spécialités × aides on/off) |
| M-07 | Espaces spécialisés par catégorie photographique | testé (modèle) | 14 espaces nommés FR/EN, outils promus ; interface : prévu |
| M-08 | Mode proposé + choix explicite prioritaire | testé | `mode_suggestions` |
| M-09 | Conseiller : exposition | testé (synthétique) | `underexposure_detected_and_fixed` |
| M-09 | Conseiller : balance des blancs | testé (synthétique) | `warm_cast_detected_and_corrected` |
| M-09 | Conseiller : écrêtage hautes lumières | testé (synthétique) | `overexposure_and_clipping_on_clipped_jpeg` |
| M-09 | Conseiller : bruit | testé (synthétique) | σ 0,030 → 0,0006 après correction |
| M-09 | Conseiller : netteté | testé (synthétique) | `defocus_detected` ; faux positifs possibles sur scènes sans contours (dit à l'utilisateur) |
| M-09 | Conseiller : flou de mouvement | testé (synthétique) | `motion_blur_detected_without_fake_fix` ; heuristique directionnelle, aucune correction inventée |
| M-09 | Conseiller : composition (horizon, tiers) | testé (synthétique) | précision de l'horizon ≈ ±0,3° sur scènes synthétiques |
| M-09 | Conseiller : dominantes colorées | testé (synthétique) | `green_tint_detected_and_corrected` |
| M-09b | Conseiller : validation sur **vraies photographies** | prévu | Seuils non calibrés sur de vraies photos |
| M-10 | Explications FR/EN de chaque constat | testé | `explanations_exist_in_both_languages_for_every_finding` |
| M-11 | Corrections prévisualisables | testé | `preview_is_small_and_leaves_source_untouched` (40 ms à 1024 px sur 24 Mpx) |
| M-12 | Correction manuelle ou automatique ; contrôle final à l'utilisateur | testé | `auto_apply_improves_combined_defects…`, `centered_subject_suggests_crop_but_never_automatically`, CLI `--only` / `--auto` obligatoire |
| M-13 | Tutoriels interactifs dans l'interface | prévu | Dépend de l'interface |
| M-14 | Exercices pratiques vérifiés par le moteur | testé | 7 exercices ; `doing_nothing_never_passes`, `solving_with_advisor_suggestion_passes_every_exercise`, `wrong_answers_fail` |
| M-15 | Photographies d'entraînement autorisées | testé | Générées par le code, libres de droits, déterministes ; collection de vraies photos autorisées : prévu |
| M-16 | Cours de photographie et de retouche | en développement | 20 leçons FR/EN ; contenu rédigé, à faire relire par des photographes |
| M-17 | Explications d'outils au survol | testé (données) | Toutes les infobulles FR/EN ; affichage : prévu |
| M-18 | Parcours progressifs par spécialité | testé | `path_is_progressive_and_specialised` |
| M-19 | Suivi de progression facultatif | testé | Désactivé par défaut ; rien n'est écrit si désactivé |
| M-20 | Recherche dans les guides | testé | Insensible casse/accents, ET logique, classement titres > mots-clés > texte |
| M-21 | Assistance contextuelle | testé | `assist.rs` : outil survolé / constat → leçon + exercice |
| M-22 | Désactivation de toutes les aides | testé | `all_aids_off_means_no_help_at_all` |
| M-23 | Profils de raccourcis type Photoshop / Lightroom / Capture One | testé (partiel) | Sans conflit, personnalisables ; couverture partielle, **à valider avec d'anciens utilisateurs** |
| M-24 | Dispositions de panneaux familières | prévu | |
| M-25 | Import de presets XMP | testé (partiel) | 7 types de réglages convertis (exposition, contraste, saturation, hautes lumières, ombres, netteté, bruit) ; les autres listés avec la raison (`unsupported_settings_are_reported…`) |
| M-26 | Migration de catalogues, métadonnées | prévu | Reconnaissance du format seulement (`detect_format`) ; voir docs/MIGRATION.md |
| M-27 | Préférences locales par défaut | testé | Fichier JSON local ; export / effacement |
| M-28 | Aucun compte requis | testé | Aucun code d'authentification ni réseau |
| M-29 | Consentement explicite par finalité, révocable | testé | `privacy.rs` : tout refusé par défaut |
| M-30 | Coût annoncé avant service payant | testé (garde) | `paid_provider_requires_cost_acknowledgement` ; services eux-mêmes : prévus |
| M-31 | Télémétrie désactivée par défaut | testé | Pas de code de télémétrie ; consentement requis |

## A — Accueil et projets · B — Bibliothèque · C — Sélection intelligente

| ID | Fonction | Statut |
|---|---|---|
| A-01…A-08 | Projets récents, séances, clients, avancement, reprise, modèles, sauvegardes, recherche globale | prévu |
| B-01 | Importation cartes/dossiers/appareils | prévu |
| B-02 | Décodage RAW (NEF, CR2, CR3, ARW, RAF, DNG), HEIC, TIFF | prévu |
| B-03 | JPEG / PNG (lecture) | intégré (CLI, via la crate `image`) — sans ICC, 8 bits |
| B-04 | Catalogues SQLite, collections, copies virtuelles | prévu |
| B-05 | Métadonnées EXIF/IPTC/XMP, mots-clés, étoiles | prévu |
| B-06 | Doublons, suivi des chemins, disques externes | prévu |
| C-01…C-05 | Flou, yeux fermés, exposition, rafales, comparaison multi-images | prévu (la détection de flou/exposition du conseiller sera réutilisée) |
| C-06 | Suppression définitive validée par un humain | prévu |

## D — Développement RAW

| ID | Fonction | Statut |
|---|---|---|
| D-01 | Dématriçage, profils d'appareils | prévu |
| D-02 | Exposition | testé (moteur) |
| D-03 | Contraste, ombres, hautes lumières | testé (moteur) — blancs/noirs : prévu |
| D-04 | Balance des blancs (gains RVB) | testé (moteur) — pipette, température/teinte réelles : prévu |
| D-05 | Saturation | testé (moteur) — vibrance : prévu |
| D-06 | Histogramme et avertissements d'écrêtage | prévu (la mesure d'écrêtage existe dans le conseiller) |
| D-07 | Clarté, texture, voile | prévu |
| D-08 | Courbes, HSL, color grading, calibration | prévu |
| D-09 | Correction d'objectif, aberrations, géométrie, perspective | prévu |
| D-10 | Netteté (masque flou 3×3) | testé (moteur) — masquage, rayon, détail : prévu |
| D-11 | Réduction du bruit (bilatéral 3×3) | testé (moteur) — 4,2 s à 24 Mpx : insuffisant, à optimiser |
| D-12 | Recadrage, rotation, redressement | testé (moteur) |
| D-13 | Masques locaux, dégradés, luminance, couleur | prévu |
| D-14 | HDR, panoramas, focus stacking | prévu |
| D-15 | Avant/après, références, instantanés | prévu |

## E — Éditeur · F — Portrait · G — IA · H — Capture

| ID | Fonction | Statut |
|---|---|---|
| E-01…E-12 | Calques, masques, modes de fusion, pinceaux, sélections, tracés, texte, formes, transformations, filtres dynamiques, historique, actions/scripts | prévu |
| E-13 | Format de projet natif versionné | prévu |
| E-14 | Import/export PSD (limites documentées) | prévu ; la signature `8BPS` est reconnue (`detect_format`) sans import |
| F-01…F-12 | Détection de personnes, peau, séparation de fréquences, Dodge & Burn, yeux, dents, cheveux, remodelage facultatif | prévu |
| G-01…G-16 | Segmentation, suppression d'objets, inpainting, ciel, relight, super-résolution, assistant en langage naturel… | prévu |
| G-17 | Gestionnaire de modèles (téléchargement facultatif, versions, licences, intégrité) | prévu ; le consentement `ModelDownload` existe |
| G-18 | Aucune photo envoyée sans consentement | testé (garde) — `ConsentLedger::require` |
| H-01…H-06 | Adaptateurs Nikon/Canon/Sony, capture connectée, Live View, presets automatiques | prévu |
| H-07 | Import Nikon D5200 NEF | prévu — **fichier NEF autorisé requis** |

## I — Presets · J — Impression · K — Galeries · L — Export

| ID | Fonction | Statut |
|---|---|---|
| I-01 | Presets natifs (piles d'`Op` sérialisables) | en développement (format sérialisable testé ; bibliothèque de presets : prévu) |
| I-02 | LUT, pinceaux, favoris, recherche | prévu |
| I-03 | Synchronisation, copie, lots, renommage, files avec pause/reprise | prévu |
| J-01…J-08 | Formats, marges, ICC, épreuvage, planches contact, albums, produits personnalisés, aperçu WYSIWYG | prévu |
| K-01…K-07 | Séances, clients, galeries privées, mots de passe, liens expirables, favoris clients, filigranes | prévu |
| K-08 | API d'intégration norelias.ca ; service Web facultatif séparé | prévu |
| L-01 | Export PNG 8 bits (CLI) | testé (e2e) |
| L-02 | Export JPEG/TIFF/WebP, 16 bits, ICC, redimensionnement, netteté de sortie, métadonnées, filigrane | prévu |
