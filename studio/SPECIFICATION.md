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

## 5. Extension — comptes, apprentissage contextuel, assistant, identité professionnelle (modules N à T)

Ces modules **complètent** les modules A–M sans en remplacer aucun. Tous restent facultatifs pour les fonctions locales.

### N — Inscription et authentification (N-A01…N-A24)
* Parcours en **4 étapes** : Compte · Studio · Expérience · Préférences (conforme à la maquette). Seuls **courriel et mot de passe sont obligatoires**, et seulement pour créer un compte ; « Utiliser sans compte » (mode local) ne demande rien.
* Champs : prénom, nom, courriel, mot de passe + confirmation (affichage/masquage, indicateur dynamique de robustesse, validation de correspondance) ; nom commercial, slogan, logo (facultatifs) ; pays, adresse, appartement/suite, ville, province/région/État, code postal ; téléphone avec indicatif ; site Web, réseaux sociaux, informations fiscales (facultatifs) ; langue, fuseau horaire, devise, unités ; niveau d'expérience, spécialités.
* Case **« Je n'ai pas encore de studio commercial »** : le nom du photographe sert de nom commercial par défaut.
* Adresses : formats propres à chaque pays ; saisie manuelle toujours possible, adresses non reconnues acceptées, fonctionnement hors connexion ; **autocomplétion internationale** via un fournisseur externe, soumise au consentement `AddressLookup` (aucune requête sans accord).
* Téléphone : validation internationale et normalisation **E.164**.
* Sécurité : hachage Argon2id, aucun mot de passe en clair, 2FA TOTP, codes de récupération, sessions, limitation des tentatives, révocation d'appareils, secrets chiffrés au repos.
* **Service en ligne** (vérification du courriel, récupération par courriel, synchronisation) : spécifié par un contrat (`ARCHITECTURE.md` §7) mais **non construit** ; l'interface doit l'indiquer comme indisponible, jamais le simuler.

### O — Niveaux (N-B01…N-B03)
Débutant · Intermédiaire · Avancé/Professionnel : changent l'accompagnement, la quantité d'explications et l'organisation initiale ; **ne verrouillent aucune fonctionnalité** ; modifiables à tout moment.

### P — Apprentissage contextuel (N-C01…N-C12)
Une **fiche pour chaque outil et chaque curseur** (nom, définition, utilité, situations, méthode, effet des réglages, erreurs fréquentes, conseils, exemple avant/après sur la **même photo**, vidéo si elle existe, exercice si pertinent, raccourci, documentation complète). Actions : Essayer maintenant · En savoir plus · Fermer · Ne plus afficher pour cet outil · Désactiver toutes les explications · Réinitialiser les tutoriels. Première utilisation : fiche complète (débutant), courte (intermédiaire), **jamais d'interruption pour un expert**. Registre centralisé : un test échoue si un outil ou un curseur n'a pas de fiche. Progression locale, synchronisation seulement avec le consentement `ProgressSync`. Aucune vidéo, aucun lien ni contenu pédagogique factice.

### Q — Académie (N-D01…N-D07)
Parcours débutant / intermédiaire / expert ; thèmes : bases de la photographie, développement RAW, portrait, calques et compositions, masques, colorimétrie, IA, impression, exportation et livraison ; photographies d'entraînement autorisées ; exercices guidés ; vidéos ; recherche ; suivi. Tout contenu portant sur un outil non intégré le déclare « prévu ».

### R — Assistant photographique (N-E01…N-E06)
Connaît les outils **réellement disponibles** et le panneau actif ; lit les réglages / la photographie **seulement avec permission** ; explique, diagnostique, guide pas à pas, propose des corrections **prévisualisables, réversibles**, et déclenche des opérations prises en charge **après validation**. N'annonce jamais une modification sans confirmation du moteur. La version v0.1 est un assistant **à règles** ; l'assistant en langage naturel libre (modèle de langage) est prévu et devra produire les mêmes propositions structurées.

### S — Identité professionnelle et filigranes (N-F01…N-F12)
Profil **Mon Studio** (source unique). Filigrane textuel (nom du studio par défaut, nom du photographe, texte libre avec `{année}`, copyright), signature, logo PNG transparent (SVG prévu), police, taille, couleur, opacité, rotation, position libre ou prédéfinie, marges, ombre, contour, répétition, adaptation portrait/paysage, aperçu en temps réel, **plusieurs modèles**, application en lot. Non destructif et facultatif. Case **« Ajouter mon filigrane »** à l'exportation : décochée ⇒ aucun filigrane ; exportation simultanée avec et sans filigrane ; **métadonnées de copyright gérées séparément** du filigrane visible.

### T — Internationalisation (N-G01…N-G10)
Textes uniquement dans un catalogue central (jamais dans les composants) ; français et anglais au lancement ; extension es, de, it, pt par ajout de fichiers ; écritures de droite à gauche prévues dans l'architecture ; formats d'adresses, indicatifs, devises, dates, fuseaux, nombres, unités métriques/impériales, taxes configurables, contenus pédagogiques traduisibles, Unicode dans les noms de fichiers et profils.

### U — Protection des données (N-H01…N-H07)
Conformité visée selon les territoires (Loi 25 au Québec, LPRPDE au Canada, RGPD en Europe) : gestion des consentements, politique de confidentialité, **export des données personnelles**, **suppression du compte**, préférences de communication, minimisation. Aucune photo de client ne sert à entraîner un modèle sans autorisation spécifique. *Cette spécification n'est pas un avis juridique : une revue par un juriste est requise avant commercialisation.*

### Écrans à fournir (N-I01…N-I09)
Inscription · Connexion · Récupération du compte · Configuration initiale · Profil professionnel · Préférences · Centre d'aide · Académie · Création de filigranes. Voir `DESIGN_SYSTEM.md` §5.

### V — Vérification obligatoire du courriel, anti-robot, courriels transactionnels, conditions (V-01…V-24)
* **Vérification** : code numérique à **6 chiffres** (aléa cryptographique, sans biais), **expiration 10 min**, **renvoi après 60 s**, anciens codes **invalidés** à chaque envoi, tentatives limitées (5 par code, 5 envois par heure et par adresse, plafonds par IP), **modification de l'adresse avant validation**, **compte non activable tant que le courriel n'est pas vérifié** (aucune session), vérification obligatoire aussi lors d'un **changement d'adresse**. Codes stockés **hachés avec un secret serveur** ; validation **côté serveur** uniquement.
* **Anti-robot** : vraie vérification **côté serveur** d'un jeton Cloudflare Turnstile, hCaptcha ou Google reCAPTCHA (au choix selon confidentialité, accessibilité, compatibilité internationale) à l'inscription, à la récupération du mot de passe et pour les connexions suspectes. **Jamais** une case à cocher simulée ; échec fermé si le fournisseur est injoignable ; message traduit et solution de secours accessible.
* **Courriels** : fournisseur transactionnel réel avec gestion des erreurs de livraison (temporaires : reprises ; définitives : rebond signalé à l'utilisateur) ; **SPF, DKIM, DMARC** sur un sous-domaine d'envoi ; modèles professionnels, responsives, multilingues, aux couleurs de Norélias : vérification, bienvenue, récupération, sécurité, facturation, suppression du compte, avis de changement d'adresse.
* **Aucun courriel fictif ni vérification simulée en production** : le serveur refuse de démarrer avec un service simulé.
* **Conditions** : acceptation **obligatoire** des conditions d'utilisation et de la politique de confidentialité, **version et date enregistrées**, nouvelle acceptation si la version change ; consentement **marketing distinct, facultatif, décoché par défaut**, avec historique daté et révocable.
* **2FA facultatif**, gestion des sessions et des appareils.
* Les textes juridiques (conditions, confidentialité) et la décision sur les fournisseurs relèvent du propriétaire du produit ; voir `ARCHITECTURE.md` §9.

## 6. Registre des exigences

Statuts : **prévu → en développement → intégré → testé → validé**. Le registre exhaustif est FEATURE_MATRIX.md.
