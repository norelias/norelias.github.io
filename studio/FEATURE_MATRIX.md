# Matrice des fonctionnalités et registre des exigences

Statuts : **prévu** · **en développement** · **intégré** (moteur réel dans le code) · **testé** (tests automatisés passent) · **validé** (vérifié sur vraies photos / vrai matériel / vrais utilisateurs).

> **Aucune ligne n'est « validée »** : aucune validation sur fichiers RAW réels, matériel ou utilisateurs n'a eu lieu.
> « Testé » signifie : tests automatisés sur images **synthétiques** générées par le code, sur Linux uniquement.
> Les preuves renvoient à des fichiers du dépôt. **229 tests** passent (`cargo test --workspace`).

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
| N-12 | Pas de dépendance commerciale imposée | testé | 104 dépendances externes, toutes permissives (LICENSES.md) |
| N-13 | Modularité, moteur indépendant de l'UI | testé | `norelias-core` sans UI ; 9 crates |
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

## Extension — modules N à U

### N — Inscription et authentification

| ID | Fonction | Statut | Preuve / remarque |
|---|---|---|---|
| N-A01 | Parcours en 4 étapes (Compte · Studio · Expérience · Préférences) : logique et validation par étape | testé | `signup.rs` (8 tests) ; **interface graphique : prévu** |
| N-A02 | Seuls courriel + mot de passe obligatoires ; mode local sans compte | testé | `only_email_and_password_are_required`, `local_only_needs_no_account` |
| N-A03 | Mot de passe : confirmation, correspondance, indicateur dynamique (5 niveaux, problèmes expliqués) | testé (logique) | `password.rs` ; bouton afficher/masquer : interface, prévu |
| N-A04 | Photographe indépendant sans studio | testé | `independent_photographer_uses_own_name` |
| N-A05 | Champs studio : nom, slogan, logo (chemin), site, réseaux, infos fiscales, devise, unités, fuseau, langue | testé (validation) | `studio_step_validates_only_what_is_filled` ; fuseaux : liste de 24 zones + contrôle de forme, **base complète prévue** |
| N-A06 | Adresses selon le pays (formats, étiquettes province/État/ZIP, codes postaux, mise en forme postale) | testé — 15 pays | CA US FR GB DE ES IT PT BE CH NL AU MX BR JP ; autres pays : format générique **sans refus** |
| N-A07 | Saisie manuelle, adresse non reconnue, hors connexion | testé | `offline_falls_back_to_manual_entry`, `empty_and_unknown_are_never_rejected_wrongly` |
| N-A08 | **Autocomplétion internationale des adresses** | **en développement** | Interface `AddressProvider` + porte de consentement + remplissage automatique : testés avec un fournisseur simulé **dans les tests uniquement**. **Aucun fournisseur réel n'est intégré** (choix, clé d'API et accord de confidentialité requis, ARCHITECTURE.md §3) |
| N-A09 | Téléphone international, format E.164 | testé | crate `phonenumber` ; `normalises_to_e164`, `rejects_invalid` |
| N-A10 | Courriel : validation de forme | testé | `emails` ; **vérification par courriel : prévu** (service en ligne) |
| N-A11 | Récupération du mot de passe | testé (local, codes) / **prévu** (courriel) | `recovery_code_is_single_use_and_disables_2fa` |
| N-A12 | Authentification à deux facteurs TOTP | testé | vecteurs RFC 6238 |
| N-A13 | Gestion des sessions, révocation d'appareils | testé (local) | `session.rs` ; liste d'appareils partagée entre postes : prévu |
| N-A14 | Limitation des tentatives | testé (local) | `limiter.rs` ; contrôle serveur : prévu |
| N-A15 | Hachage Argon2id, secrets chiffrés au repos | testé | `crypto.rs` |
| N-A16 | Coffre de l'OS pour les secrets | prévu | |
| N-A17 | Mode local hors connexion après configuration | testé | aucun code réseau |
| N-A18 | Service d'authentification en ligne | **prévu** (non construit) | contrat : ARCHITECTURE.md §7 |

### O — Niveaux · P — Apprentissage contextuel · Q — Académie

| ID | Fonction | Statut | Preuve / remarque |
|---|---|---|---|
| N-B01 | Trois niveaux, modifiables, sans verrouillage | testé (modèle) | `no_mode_or_specialty_ever_removes_a_tool`, `mode_override` |
| N-C01 | Registre central des fonctions documentées | testé | `every_available_tool_has_a_complete_card_and_nothing_else_does` |
| N-C02 | Fiche par outil : définition, utilité, situations, méthode, erreurs, conseils (FR/EN) | testé (12 fiches) | `help.rs` ; **contenu à faire relire par des photographes** |
| N-C03 | Documentation de **chaque curseur** du moteur | testé | `every_slider_of_the_engine_is_documented` (15 curseurs, liés aux champs réels de `Op`) |
| N-C04 | Exemple avant/après sur la même photo | testé (10 outils) | `examples_use_the_same_photograph_and_really_differ`, `rotate_example_really_straightens_the_horizon` |
| N-C05 | Tutoriel vidéo | **prévu — aucune vidéo n'existe** | `no_fake_media_or_links` : la fiche n'en annonce jamais |
| N-C06 | Exercice interactif lié | testé | 7 exercices |
| N-C07 | Raccourci clavier dans la fiche | testé (1 outil : Recadrer) | les autres outils n'ont pas de raccourci défini |
| N-C08 | Lien vers la documentation complète | testé | pages Markdown générées, lien vérifié (`full_documentation_pages_are_generated…`) |
| N-C09 | Actions : Essayer · En savoir plus · Fermer · Ne plus afficher · Désactiver tout · Réinitialiser | testé (logique) | `mute_disable_and_reset` ; boutons : interface, prévu |
| N-C10 | Première utilisation adaptée au niveau, jamais d'interruption d'un expert | testé | `first_use_policy_by_level` |
| N-C11 | Progression enregistrée localement ; synchronisation seulement avec consentement | testé (garde) | `state_persists_locally_and_sync_requires_consent` ; **transport de synchronisation : prévu** |
| N-C12 | Bouton d'aide contextuelle et infobulles accessibles | prévu (interface) | données prêtes |
| N-D01 | Parcours débutant / intermédiaire / expert | testé (parcours débutant→avancé par niveau et spécialité) | `path_is_progressive_and_specialised` |
| N-D02 | Cours : bases, RAW, portrait, calques, masques, colorimétrie, IA, impression, export | **en développement** | 20 leçons (bases, retouche, spécialités) ; **RAW, calques, masques, colorimétrie, IA, impression : aucun cours** car aucun outil intégré |
| N-D03 | Bibliothèque de photographies d'entraînement autorisées | en développement | 7 images procédurales libres de droits ; photos réelles autorisées : prévu |
| N-D04 | Vidéos | **prévu — aucune** | |
| N-D05 | Recherche, suivi | testé | |

### R — Assistant photographique

| ID | Fonction | Statut | Preuve / remarque |
|---|---|---|---|
| N-E01 | Explique les outils **réels** ; dit « prévu » pour les autres | testé | `explains_real_tools…`, `capabilities_list_exactly_the_registry` |
| N-E02 | Connaît le panneau actif | testé | `active_tool` |
| N-E03 | Lit la photo seulement avec permission | testé | `never_looks_at_the_photo_without_permission` |
| N-E04 | Propose des corrections prévisualisables, guide pas à pas | testé | `proposal_preview_confirmation_apply_undo_redo` |
| N-E05 | Applique après validation ; réversible ; n'annonce jamais sans confirmation du moteur | testé | `a_noop_is_reported_as_nothing_changed` |
| N-E06 | **Compréhension du langage naturel libre** | **prévu** | v0.1 : vocabulaire fixe FR/EN par règles, aucun modèle de langage |

### S — Identité professionnelle et filigranes

| ID | Fonction | Statut | Preuve / remarque |
|---|---|---|---|
| N-F01 | Profil « Mon Studio » (stockage, nom de marque, titulaire du copyright) | testé | `profile.rs` ; écran : prévu |
| N-F02 | Filigrane textuel : nom du studio par défaut, photographe, texte libre, copyright, année automatique | testé | `text_is_resolved_from_studio_with_placeholders` |
| N-F03 | Logo PNG transparent | testé | `transparent_logo_keeps_its_transparency` ; **SVG : prévu** (refus explicite aujourd'hui) |
| N-F04 | Signature manuscrite | en développement | style « Élégant » (Liberation Serif Italic) ou image importée ; **saisie/tracé à la main : prévu** |
| N-F05 | Police, taille, couleur, opacité, rotation | testé | 2 polices embarquées (Inter, Liberation Serif Italic) ; choix d'autres polices : prévu |
| N-F06 | Positions prédéfinies (grille 3×3) et libre, marges | testé | `every_preset_lands_where_it_says`, `margin_moves_the_mark_inward` |
| N-F07 | Ombre, contour, répétition | testé | `shadow_outline_and_tiling_add_pixels` |
| N-F08 | Adaptation portrait/paysage | testé | `portrait_and_landscape_overrides` |
| N-F09 | Aperçu en temps réel | testé (moteur) | 2–14 ms à 1000 px ; fidèle à l'export ; affichage : interface, prévu |
| N-F10 | Plusieurs modèles enregistrés | testé | `templates.rs` |
| N-F11 | Application en lot | testé | `batch_applies_to_every_image_and_isolates_failures` |
| N-F12 | Non destructif, facultatif ; case **« Ajouter mon filigrane »** : décochée ⇒ aucun filigrane | testé | `unchecked_box_means_no_watermark_at_all` (octet pour octet) |
| N-F13 | Export simultané avec et sans filigrane | testé | `dual_export_makes_both_versions` |
| N-F14 | Copyright invisible séparé du filigrane visible | testé (PNG iTXt, JPEG `.xmp` annexe) | `copyright_metadata_is_independent…` ; **XMP/EXIF intégré au JPEG : prévu** |

### T — Internationalisation · U — Protection des données

| ID | Fonction | Statut | Preuve / remarque |
|---|---|---|---|
| N-G01 | Catalogue central ; aucun texte dans les composants | testé (nouveaux modules) | `catalogs_are_in_sync` ; **anciens textes (conseiller, Académie, questionnaire) encore intégrés au code : migration prévue** |
| N-G02 | Français et anglais | testé (catalogue) | 74 clés FR/EN |
| N-G03 | Espagnol, allemand, italien, portugais | **prévu** | formats de nombres prêts ; **aucune traduction** |
| N-G04 | Droite-à-gauche | en développement | détection de direction testée ; **miroir de l'interface : prévu** |
| N-G05 | Nombres, devises, dates | testé | FR/EN (+ séparateurs es/de/it/pt) ; dates longues FR/EN seulement |
| N-G06 | Unités métriques/impériales | testé | `units.rs` |
| N-G07 | Taxes configurables | testé | aucun taux imposé ; préréglages Québec/Ontario **à faire valider par un comptable** |
| N-G08 | Fuseaux horaires | en développement | forme + 24 zones courantes |
| N-G09 | Unicode dans fichiers et profils | testé | `resize_and_unicode_names_and_hostile_names`, profils Unicode |
| N-G10 | Contenus pédagogiques traduisibles | testé (FR/EN) | |
| N-H01 | Consentements (8 finalités), révocables | testé | `privacy.rs` |
| N-H02 | Export des données personnelles | testé (compte) | `export_contains_no_secrets` ; export complet du profil/studio/catalogue : à regrouper |
| N-H03 | Suppression du compte | testé (local) | `AccountStore::delete` ; suppression côté serveur : prévu |
| N-H04 | Politique de confidentialité, préférences de communication | **prévu** | texte juridique à rédiger et faire relire |
| N-H05 | Minimisation | testé | seuls courriel + mot de passe requis |
| N-H06 | Aucune photo de client pour l'entraînement sans autorisation | testé (par absence) | aucun entraînement n'existe ; case par séance : prévue |
| N-H07 | Conformité Loi 25 / LPRPDE / RGPD | **non évaluée** | revue juridique requise |

### Écrans (N-I01…N-I09)
Inscription, connexion, récupération, configuration initiale, profil professionnel, préférences, centre d'aide, académie, création de filigranes : **spécifiés** (DESIGN_SYSTEM.md §5.3), **non construits** (aucune interface graphique).

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
