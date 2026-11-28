# Cahier des charges — Norélias Studio Pro

Version 0.1. Les identifiants (A-xx, M-xx…) sont repris dans [FEATURE_MATRIX.md](FEATURE_MATRIX.md) et [TEST_PLAN.md](TEST_PLAN.md).

## 1. Mission

Application desktop (Windows, macOS) réunissant tout le flux de travail d'un photographe : importation,
sélection, développement RAW, composition multicouche, portrait, IA locale, capture connectée, impression,
galeries clients, exportation. Identité propre : aucune interface, icône ni ressource propriétaire tierce.

## 2. Exigences non négociables

| ID | Exigence |
|---|---|
| N-01 | Application desktop réelle, pas uniquement un site Web |
| N-02 | Outils classiques utilisables sans Internet |
| N-03 | Interface entièrement traduite français / anglais |
| N-04 | Fichiers originaux jamais modifiés ni écrasés |
| N-05 | Retouche non destructive (opérations enregistrées à part) |
| N-06 | Traitement colorimétrique en virgule flottante, conversions explicites |
| N-07 | Accélération GPU avec repli CPU |
| N-08 | RAW haute résolution ; catalogues de dizaines de milliers de photos |
| N-09 | Écrans haute résolution |
| N-10 | Sauvegarde automatique et récupération après plantage |
| N-11 | Aucun bouton factice, aucun résultat simulé présenté comme fonctionnel |
| N-12 | Aucune dépendance commerciale imposée sans justification (voir LICENSES.md) |
| N-13 | Architecture modulaire, documentée, testable ; moteur indépendant de l'interface |
| N-14 | Confidentialité des photographies clients |

**Règle de déclaration** : une fonctionnalité n'est déclarée « intégrée » que si son moteur réel est dans le
code, « testée » que si des tests automatisés passent, « validée » que si elle a été validée sur de vraies
photographies / du vrai matériel / par de vrais utilisateurs.

## 3. Modules A–L

Repris du brief initial ; le détail de chaque exigence et son statut est dans FEATURE_MATRIX.md.
A Accueil et projets · B Bibliothèque · C Sélection intelligente · D Développement RAW · E Éditeur avancé ·
F Studio portrait · G Studio IA · H Capture studio · I Presets et automatisation · J Impression et création ·
K Galeries et clients · L Exportation.

Contraintes transversales rappelées : suppression définitive toujours validée par un humain (C) ;
aucune photo envoyée à un service externe sans consentement explicite, coût annoncé avant tout service payant (G, K) ;
capture à distance non promise sans validation matérielle, test d'importation Nikon D5200 NEF obligatoire (H) ;
fonctions PSD incompatibles documentées (E) ; service Web de galeries séparé du module local (K).

## 4. Module M — Personnalisation et apprentissage (obligatoire)

### M1 Questionnaire initial (M-01…M-03)

Au premier lancement, collecte (toutes réponses facultatives) : niveau en photographie ; niveau en retouche ;
spécialités ; logiciels déjà utilisés ; marque et modèle de l'appareil ; objectifs ; besoins en IA ; besoins
en impression et livraison ; préférences d'accompagnement ; langue et apparence ; préférences de stockage et de confidentialité.

* Le questionnaire peut être **passé** à tout moment ; passer ne bloque rien et ne redemande jamais automatiquement.
* **Toutes** les réponses sont modifiables plus tard (menu Aide → Questionnaire, et préférences).
* Les réponses ne servent qu'à adapter l'interface ; elles ne quittent jamais l'ordinateur (M-14).

### M2 Espaces de travail adaptatifs (M-04…M-08)

Trois modes — débutant, intermédiaire, expert — sur **le même moteur de traitement**. Ils adaptent :
visibilité des panneaux, complexité des réglages (essentiels / standard / complets), explications
(simples / concises / aucune), indications de raccourcis, recommandations (proactives / à la demande / pastille), tutoriels proposés.

* **Aucun outil n'est jamais supprimé** selon le niveau : la visibilité a trois états (déployé, replié, à la demande
  via recherche d'outils) et le type n'a volontairement pas d'état « supprimé ».
* Le mode proposé découle des niveaux déclarés ; le choix explicite de l'utilisateur l'emporte toujours.
* Espaces spécialisés par catégorie (portrait, mariage, famille, nouveau-né, paysage, animalier, sport, événementiel,
  produit, immobilier, rue, studio, scolaire) : ils *promeuvent* des outils, sans en exclure.
* Un outil non intégré est affiché comme « prévu », jamais comme actif.

### M3 Conseiller photographique (M-09…M-12)

Analyse locale des caractéristiques techniques : exposition, balance des blancs, écrêtage des hautes lumières,
bruit, netteté, flou de mouvement, composition (horizon, règle des tiers), dominantes colorées.

* Chaque constat est **expliqué** (FR/EN) avec sa mesure, ses limites et les cas où l'ignorer est légitime.
* Chaque correction est une suite d'opérations réelles du moteur, **prévisualisable** (aperçu réduit) avant application.
* Application **manuelle** (choix des corrections) ou **automatique** (uniquement les corrections « sûres » ;
  jamais les choix créatifs comme le recadrage ni le renforcement).
* Quand aucune correction honnête n'existe (flou de mouvement), aucune n'est inventée : conseil de prise de vue seulement.
* L'utilisateur garde le contrôle final : l'image source n'est jamais modifiée, l'application produit une nouvelle image / pile d'opérations.

### M4 Académie Norélias (M-13…M-22)

Tutoriels interactifs dans l'interface ; exercices pratiques ; photographies d'entraînement autorisées ;
cours de photographie et de retouche ; explications d'outils au survol ; parcours progressifs par spécialité ;
suivi de progression facultatif (désactivé par défaut, local, effaçable) ; recherche dans les guides ;
assistance contextuelle ; **désactivation possible de toutes les aides**.

* Les exercices sont vérifiés par le vrai moteur et le vrai conseiller (pas de réponse codée en dur).
* Les photographies d'entraînement sont **générées par le code** : aucun droit tiers, redistribuables.
* Les leçons mentionnant un outil non intégré le disent explicitement.

### M5 Compatibilité avec les anciens logiciels (M-23…M-26)

* Profils de raccourcis familiers (type Photoshop, Lightroom, Capture One) fondés sur des conventions d'usage ;
  **aucune** icône, disposition, nom ni texte propriétaire reproduit. Couverture partielle documentée.
* Dispositions de panneaux familières : prévu (interface graphique).
* Migration documentée ([docs/MIGRATION.md](docs/MIGRATION.md)) : presets XMP convertis en opérations réelles pour les
  réglages équivalents, **rapport** listant chaque réglage non converti et pourquoi ; catalogues : reconnaissance du
  format puis import prévu ; métadonnées XMP/IPTC : prévu. Approximations visuelles, pas d'équivalence pixel à pixel.

### M6 Confidentialité (M-27…M-31)

* Préférences et profil **locaux par défaut** (fichier JSON), **aucun compte requis** pour les fonctions locales.
* **Aucune photographie ni habitude d'usage transmise sans consentement explicite**, par finalité, journalisé, révocable.
  Un service payant exige la confirmation du coût avant utilisation.
* Statistiques d'usage et rapports de plantage : désactivés par défaut.
* Suppression des données d'intégration locales sur demande (écrasement puis suppression, avec les limites du SSD indiquées).

## 5. Registre des exigences

Statuts : **prévu → en développement → intégré → testé → validé**. Le registre exhaustif est FEATURE_MATRIX.md.
