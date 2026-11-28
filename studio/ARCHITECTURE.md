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
norelias-core        pixels f32 linéaires · Op (10) · couleur sRGB · images synthétiques · rééchantillonnage
   ▲
norelias-advisor     mesures + constats + corrections prévisualisables
   ▲                          ▲
norelias-onboarding ◄─────────┘ (core seulement)   profil · store · privacy · tools · workspace · shortcuts · compat · questionnaire
   ▲
norelias-academy     leçons · exercices · recherche · infobulles · progression · aide contextuelle · registre d'aide (fiches, exemples avant/après)
   ▲                 ▲
   │      norelias-assistant   assistant à règles : intentions · propositions · confirmation du moteur · annuler/rétablir
norelias-i18n        catalogue central FR/EN · nombres, monnaies, dates · unités · taxes · RTL
norelias-account     mots de passe · Argon2id · TOTP · sessions · limitation · téléphone E.164 · adresses · inscription 4 étapes
norelias-studio      « Mon Studio » · filigrane (texte, logo, ombre, contour, répétition) · exportation · modèles
norelias-server      logique serveur : vérification du courriel, CAPTCHA, courriels, SPF/DKIM/DMARC, conditions, récupération (sans client réseau)
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

### ADR de l'extension (comptes, i18n, assistant, filigrane)

| ADR | Décision | Justification | Statut |
|---|---|---|---|
| 013 | **Deux couches d'authentification** : locale (réelle, hors ligne) + service en ligne (contrat `AuthBackend`, §7) | Le mode local doit fonctionner sans compte ni réseau ; courriel/synchronisation exigent un serveur | Locale : adoptée, testée ; en ligne : **non construite** |
| 014 | Mots de passe : **Argon2id** (19 Mio, t=2, p=1, minimum OWASP), sel aléatoire, jamais en clair | Résistance GPU ; paramètres stockés avec le hachage pour migration | Adoptée |
| 015 | 2FA : **TOTP RFC 6238** (SHA-1, 6 chiffres, 30 s), anti-rejeu ; secret chiffré (ChaCha20-Poly1305) par une clé dérivée du mot de passe ; codes de récupération hachés | Pas de coffre de l'OS disponible ici ; un changement de mot de passe rechiffre, une récupération désactive le 2FA | Adoptée, vecteurs RFC testés ; **coffre Keychain/Credential Manager : prévu** |
| 016 | Jetons de session : 256 bits aléatoires, seul le SHA-256 est stocké ; expiration d'inactivité 30 j, absolue 90 j | Fuite du fichier ≠ session volée | Adoptée |
| 017 | Téléphone : crate `phonenumber` (métadonnées libphonenumber, Apache-2.0) | Validation internationale réelle plutôt que regex maison | Adoptée |
| 018 | Adresses : formats détaillés pour 15 pays, format générique **sans refus** ailleurs ; **autocomplétion = trait `AddressProvider` derrière une porte de consentement** | Ne jamais refuser une adresse réelle ; ne rien envoyer sans accord | Trait et porte : testés ; **fournisseur : non choisi** (voir ci-dessous) |
| 019 | i18n : catalogue central JSON, repli langue → anglais → clé, audit de cohérence | Aucun texte dans les composants ; ajout d'une langue = un fichier | Adoptée ; migration des anciens textes : **en cours** |
| 020 | Assistant **à règles** avec propositions structurées validées par le moteur | Fiable, hors ligne, vérifiable ; la même interface accueillera un modèle de langage | Adoptée |
| 021 | Filigrane = **description** appliquée à l'export/aperçu ; rendu de texte `fontdue`, polices Inter et Liberation Serif Italic (SIL OFL) embarquées | Non destructif ; polices libres embarquables | Adoptée |
| 022 | Métadonnées de copyright : PNG → iTXt UTF-8 ; JPEG → fichier annexe `.xmp` | Séparation du filigrane visible ; l'écriture EXIF/XMP intégrée au JPEG demande un écrivain dédié | PNG/XMP annexe adoptés ; **JPEG intégré : prévu** |

**Fournisseur d'autocomplétion d'adresses — décision à prendre** (non vérifié ici : tarifs, conditions et licences changent ; à confirmer sur les sites des fournisseurs). Critères : couverture internationale, conditions d'usage du stockage des résultats, tarif, protection de la vie privée (hébergement, journaux), possibilité d'auto-hébergement. Pistes : Photon/Nominatim (données OpenStreetMap, auto-hébergeables), Geoapify, Mapbox, Google Places, Canada Post AddressComplete (Canada seulement). Il faudra aussi une clé d'API stockée dans le coffre de l'OS (jamais dans le dépôt ni le profil) et un déclencheur avec délai (≥ 300 ms) pour limiter les requêtes. Tant que ce choix n'est pas fait, l'interface affiche la saisie manuelle et n'annonce aucune suggestion.

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

## 7. Contrat du service en ligne (`AuthBackend`) — **logique implémentée dans `norelias-server`, serveur HTTP non déployé**

Le client local fonctionne sans ce service. Quand il existera, il devra offrir (TLS, limitation de débit côté serveur, journalisation minimale) :

| Fonction | Contrat | Notes |
|---|---|---|
| Création de compte | `POST /v1/accounts` {courriel, verificateur d'authentification} → 202 | Le serveur ne reçoit pas le mot de passe en clair (protocole PAKE/OPAQUE à évaluer) ; courriel de vérification avec jeton à usage unique, expiration 24 h |
| Vérification du courriel | `POST /v1/accounts/verify` {jeton} | `email_verified` passe à vrai seulement ici |
| Connexion | `POST /v1/sessions` + 2FA | Limitation de débit serveur **en plus** de la limitation locale |
| Récupération | `POST /v1/recovery` → courriel à usage unique | Complète les codes de récupération locaux |
| Appareils | `GET/DELETE /v1/devices` | Révocation immédiate des jetons |
| Synchronisation | `PUT /v1/sync/{profil, progression}` | **Uniquement** avec les consentements `AccountSync` / `ProgressSync` ; données chiffrées côté client |
| Données personnelles | `GET /v1/me/export`, `DELETE /v1/me` | Droits d'accès, de portabilité et d'effacement |

Jusqu'à sa construction, `AccountRecord::email_verified` reste **faux** et l'interface affiche « non disponible en mode local » (clé `auth.email_verification_unavailable`).

## 8. Performances mesurées — filigrane (24 Mpx, CPU mono-thread, `--release`)

| Opération | Aperçu 1000 px | 24 Mpx |
|---|---|---|
| Texte simple | 14 ms | 0,23 s |
| Ombre + contour + rotation | 6 ms | 0,75 s |
| Répété sur toute l'image | 2 ms | 0,23 s |
| Réduction 24 Mpx → 2048 px (moyenne d'aire) | — | 0,16 s |

L'aperçu en temps réel est donc réaliste à 1000 px. Un aperçu fidèle : les tailles sont relatives au côté court, test `preview_matches_export_proportions`.

## 9. Mise en production de la vérification du courriel — ce qui est fait, ce qui manque

### 9.1 Fait et testé (`norelias-server`, 54 tests dont 20 parcours complets)
Cycle de vie des codes · inscription/activation/connexion/changement d'adresse/récupération/suppression · CAPTCHA côté serveur (3 fournisseurs : requêtes, réponses, échec fermé) · file d'envoi avec reprises et rebonds · 13 modèles de courriel FR/EN · générateurs et validateurs SPF/DKIM/DMARC · conditions et consentement marketing · garde « rien de simulé en production » · limites par IP · anti-énumération · anti-pré-détournement.

### 9.2 Décisions d'architecture de l'extension « vérification »
| ADR | Décision | Justification |
|---|---|---|
| 023 | Codes : 6 chiffres, HMAC-SHA256 avec secret serveur, 5 essais par code, TTL 10 min, renvoi ≥ 60 s | Un code de 10⁶ valeurs n'est sûr que si les essais sont limités ; le HMAC empêche de retrouver un code depuis une base volée |
| 024 | L'activation exige **code + jeton d'inscription** remis au client | Empêche qu'un tiers inscrive l'adresse d'autrui puis profite de son code (« pré-détournement ») |
| 025 | CAPTCHA vérifié **avant** toute logique de compte ; **échec fermé** | Une panne du fournisseur ne doit pas ouvrir l'inscription ; message `captcha.unavailable` + soutien |
| 026 | Interfaces `EmailSender` / `CaptchaVerifier` / `HttpTransport`, garde d'environnement | Aucun service fictif ne peut démarrer en production ; le fournisseur reste interchangeable |
| 027 | Récupération par courriel **seul** refusée si le 2FA est actif (un code de récupération est aussi exigé) | Un accès au courriel ne doit pas contourner le deuxième facteur |
| 028 | File d'envoi **en mémoire** : les codes ne sont jamais écrits sur disque | Un plantage coûte un renvoi, pas une fuite de codes |

### 9.3 Ce qui manque pour une vérification **réelle** — et qui dépend de vous
Rien de ceci ne peut être fait ni vérifié depuis cet environnement (pas de serveur hébergé, pas de compte chez un fournisseur, pas d'accès au DNS de norelias.ca, pas de clés).

| Élément | Décision / action requise | Notes |
|---|---|---|
| **Fournisseur de courriels transactionnels** | Choisir et ouvrir un compte ; fournir une clé d'API (stockée comme secret du serveur, jamais dans le dépôt) | Critères : délivrabilité, hébergement des données (Canada/UE, Loi 25/RGPD), coût, gestion des rebonds, support de SPF/DKIM personnalisés. Constructeurs de requêtes prêts pour **Postmark** et **Resend** (format écrit de mémoire d'après leur documentation, **à valider par un envoi réel**) ; d'autres fournisseurs (Amazon SES, Mailgun, SendGrid…) = une fonction `build_request` à ajouter |
| **Domaine d'envoi + DNS** | Créer un sous-domaine d'envoi (ex. `mail.norelias.ca`), publier SPF, DKIM (clé fournie par le fournisseur) et DMARC chez le gestionnaire DNS | Générer les valeurs : `norelias mail dns <domaine> --include … --selector … --key … --rua …`. Commencer DMARC en `p=none`, lire les rapports, puis `quarantine`, puis `reject`. Vérifier : `dig TXT <nom>`, outils du fournisseur, test d'envoi vers Gmail/Outlook |
| **CAPTCHA** | Choisir Turnstile, hCaptcha ou reCAPTCHA ; créer le site ; fournir clé de site (client) et **clé secrète (serveur)** | Turnstile : sans énigme visuelle dans la plupart des cas, bonne confidentialité ; hCaptcha : confidentialité, option d'accessibilité ; reCAPTCHA : très répandu mais envoie des données à Google. **Tous transmettent l'adresse IP à un tiers : à mentionner dans la politique de confidentialité**. Chine continentale : disponibilité variable selon le fournisseur (non vérifié) |
| **Serveur HTTP** | Héberger `norelias-server` derrière HTTPS : routes, `HttpTransport` réel (client HTTP), stockage SQL transactionnel (l'état actuel est un fichier JSON mono-processus), journaux sans secrets, sauvegardes | Le crate ne contient volontairement aucun client réseau |
| **Textes juridiques** | Rédiger et faire relire conditions d'utilisation et politique de confidentialité ; fournir leurs identifiants de version | Le code enregistre la version et la date d'acceptation, pas le texte |
| **Marketing** | Si des courriels marketing sont un jour envoyés : modèle, lien de désabonnement fonctionnel, adresse postale, vérification du consentement (LCAP/CASL, RGPD) | **Aucun courriel marketing n'existe ni n'est préparé** ; seule la case de consentement est gérée |
| **Facturation** | Le modèle de reçu existe ; **aucun système de facturation** ne le déclenche | |
| **Logo dans les courriels** | Fournir un logo hébergé en HTTPS (les images sont volontairement absentes aujourd'hui) | À peser contre le suivi d'ouverture et le blocage d'images |

### 9.4 Vérification d'acceptation à faire avec les vrais services (liste)
1. Inscription réelle → courriel reçu en < 1 min dans Gmail, Outlook, Apple Mail, une boîte professionnelle (pas dans les indésirables) ; en-têtes : `SPF=pass`, `DKIM=pass`, `DMARC=pass`.
2. Code expiré, renvoi, mauvais code ×5, changement d'adresse : mêmes résultats que les tests.
3. CAPTCHA : jeton valide accepté, jeton rejoué refusé, jeton d'un autre site refusé, panne simulée (clé invalide) → `captcha.unavailable`.
4. Rebond : adresse inexistante → message `email.undeliverable` ; plainte spam / désabonnement traités.
5. Rendu des 13 modèles dans les principaux clients (mode sombre, mobile, lecteur d'écran), FR et EN.
6. Charge : limites par IP derrière un répartiteur (adresse IP réelle du client, en-têtes de proxy de confiance).
