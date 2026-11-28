# Feuille de route

Le découpage en phases ne supprime aucune fonctionnalité des phases suivantes. Statuts détaillés : FEATURE_MATRIX.md.

## Phase 1 — Fondations (en cours)

Livré (v0.1, testé) : moteur d'image f32 linéaire et 10 opérations ; conseiller photographique ; profil local, questionnaire,
espaces de travail, consentements ; Académie (20 leçons, 7 exercices) ; import de presets XMP partiel ; CLI de bout en bout.

Extension livrée (testée, v0.1) : compte local (Argon2id, TOTP, codes de récupération, sessions, limitation), validation d'adresses (15 pays) et de
téléphones E.164, inscription en 4 étapes (logique), catalogue i18n FR/EN, registre d'aide (12 fiches, 15 curseurs, exemples avant/après), assistant à règles,
filigrane complet et exportation avec/sans filigrane.

Vérification du courriel (testée avec des doubles) : cycle de vie des codes, activation obligatoire, CAPTCHA côté serveur, file d'envoi, 13 modèles FR/EN, SPF/DKIM/DMARC (générateurs/validateurs), conditions et consentement marketing, récupération, 2FA, suppression.

**Bloqué par des décisions ou accès du propriétaire** (ARCHITECTURE.md §9.3) :
- [ ] Choisir le **fournisseur de courriels transactionnels**, ouvrir le compte, fournir la clé d'API
- [ ] Créer le **sous-domaine d'envoi** et publier **SPF / DKIM / DMARC** (valeurs : `norelias mail dns …`), puis vérifier la délivrabilité
- [ ] Choisir le **CAPTCHA** (Turnstile / hCaptcha / reCAPTCHA), fournir clé de site et clé secrète ; l'inscrire dans la politique de confidentialité
- [ ] Héberger le **serveur HTTP** (HTTPS, client HTTP réel, base SQL transactionnelle, IP client fiable derrière le proxy)
- [ ] Rédiger les **conditions d'utilisation** et la **politique de confidentialité** (revue juridique) et fournir leurs identifiants de version
- [ ] Exécuter les **tests d'acceptation avec les services réels** (§9.4) et tester les modèles dans Gmail, Outlook, Apple Mail
- [ ] Construire les écrans : vérification du code, CAPTCHA, conditions (DESIGN_SYSTEM.md §5.5)
- [ ] Facturation (déclencheur des reçus) et, si voulu, infrastructure marketing conforme (désabonnement)

Reste pour clore la phase 1 :
- [ ] **Choisir et intégrer le fournisseur d'autocomplétion d'adresses** (critères : ARCHITECTURE.md §3) ; clé d'API dans le coffre de l'OS
- [ ] **Construire le service en ligne** (contrat : ARCHITECTURE.md §7) : vérification du courriel, récupération, appareils, synchronisation chiffrée, export/suppression côté serveur
- [ ] Coffre de l'OS (Keychain / Credential Manager) pour les secrets 2FA et les clés
- [ ] Migrer les textes du conseiller, de l'Académie et du questionnaire vers le catalogue i18n
- [ ] Écrans : inscription, connexion, récupération, configuration initiale, Mon Studio, préférences, centre d'aide, Académie, filigranes (DESIGN_SYSTEM.md §5.3)
- [ ] Rédiger la politique de confidentialité et la faire relire (Loi 25, LPRPDE, RGPD) ; nommer un responsable de la protection des renseignements
- [ ] Prototype de la coque Tauri + React (ADR-004) avec les écrans du module M (questionnaire, espaces, conseiller, Académie)
- [ ] Gestion des couleurs : Little CMS, profils ICC, 16 bits (ADR-003)
- [ ] Décodage RAW (LibRaw) — **tests avec des NEF Nikon D5200 autorisés**, plus CR3/ARW/RAF/DNG
- [ ] Catalogue SQLite avec migrations, importation, vérification d'intégrité, reconnexion de disques
- [ ] Format de projet natif versionné ; sauvegarde automatique et journal de récupération
- [ ] Rendu par tuiles, pyramide d'aperçus, annulation, multithreading (réduction du bruit 4,2 s → cible < 0,5 s à 24 Mpx)
- [ ] Calibration du conseiller sur un jeu de **vraies photographies** autorisées

## Phase 1 bis — Apprentissage et contenu (parallèle)
Relecture des 20 leçons et 12 fiches par des photographes ; vraies photographies d'entraînement autorisées ; **vidéos** (aucune n'existe : l'interface n'en annoncera pas avant) ;
parcours RAW / calques / masques / colorimétrie / IA / impression **au fur et à mesure** que les outils correspondants sont intégrés ; traductions es, de, it, pt ; miroir droite-à-gauche ; logos SVG ; signature tracée à la main ; XMP/EXIF intégré aux JPEG ; assistant en langage naturel libre (modèle local ou service avec consentement et coût annoncé).

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
