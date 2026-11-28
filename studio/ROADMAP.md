# Feuille de route

Le découpage en phases ne supprime aucune fonctionnalité des phases suivantes. Statuts détaillés : FEATURE_MATRIX.md.

## Phase 1 — Fondations (en cours)

Livré (v0.1, testé) : moteur d'image f32 linéaire et 10 opérations ; conseiller photographique ; profil local, questionnaire,
espaces de travail, consentements ; Académie (20 leçons, 7 exercices) ; import de presets XMP partiel ; CLI de bout en bout.

Reste pour clore la phase 1 :
- [ ] Prototype de la coque Tauri + React (ADR-004) avec les écrans du module M (questionnaire, espaces, conseiller, Académie)
- [ ] Gestion des couleurs : Little CMS, profils ICC, 16 bits (ADR-003)
- [ ] Décodage RAW (LibRaw) — **tests avec des NEF Nikon D5200 autorisés**, plus CR3/ARW/RAF/DNG
- [ ] Catalogue SQLite avec migrations, importation, vérification d'intégrité, reconnexion de disques
- [ ] Format de projet natif versionné ; sauvegarde automatique et journal de récupération
- [ ] Rendu par tuiles, pyramide d'aperçus, annulation, multithreading (réduction du bruit 4,2 s → cible < 0,5 s à 24 Mpx)
- [ ] Calibration du conseiller sur un jeu de **vraies photographies** autorisées

## Phase 2 — Développement RAW complet
Courbes, HSL, color grading, calibration, objectifs, masques (dégradés, luminance, couleur), presets/LUT, lots, files avec
pause/reprise, exportation multi-formats, HDR/panoramas/focus stacking, GPU (`wgpu`) avec repli CPU.

## Phase 3 — Éditeur multicouche
Calques pixel/vectoriels/réglage, masques, modes de fusion, outils manuels, objets dynamiques, historique, actions/scripts, import/export PSD limité et documenté.

## Phase 4 — Portrait et IA locale
Détection de personnes, retouche naturelle de la peau, segmentation, suppression d'objets, ONNX Runtime, gestionnaire de modèles
(téléchargement facultatif, versions, licences, intégrité). Académie : leçons et exercices portrait.

## Phase 5 — IA générative, capture, impression, galeries
Inpainting/outpainting, assistant en langage naturel (opérations structurées, réversibles), capture connectée (validation
matérielle avant toute promesse), impression WYSIWYG + ICC, service Web facultatif de galeries et API norelias.ca.

## Phase 6 — Qualité commerciale
Optimisation (profils 32 Go + RTX 3060 / sans GPU), accessibilité, tests matériels Windows/macOS, installateurs signés, documentation, tests utilisateurs
(dont validation des profils de raccourcis par d'anciens utilisateurs).
