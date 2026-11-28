# Sécurité et confidentialité

## Garanties implémentées et testées (v0.1)

| Garantie | Mécanisme | Test |
|---|---|---|
| Un original n'est jamais écrasé | Le moteur ne modifie pas sa source ; la CLI refuse une sortie existante ou identique à l'entrée | `never_overwrites_original_or_existing_file`, `source_is_never_modified` |
| Aucune photo ni statistique ne sort sans consentement | Aucun code réseau ; `ConsentLedger::require(purpose)` obligatoire ; tout refusé par défaut | `nothing_is_allowed_by_default`, `no_networking_crate_in_dependency_graph` |
| Service payant annoncé | `grant()` exige la confirmation du coût | `paid_provider_requires_cost_acknowledgement` |
| Pas de compte requis | Aucune authentification dans le code local | e2e `first_run_is_local_private_and_skippable` |
| Profil jamais perdu en cas de coupure | tmp → fsync → rename, copie `.bak`, quarantaine du fichier corrompu (jamais supprimé) | `no_temporary_file_is_left_behind`, `corrupted_file_is_quarantined…` |
| Fichier d'une version plus récente non écrasé | Refus de charger, fichier intact | `newer_schema_is_refused_and_left_untouched` |
| Saisies validées | Longueur et caractères de contrôle refusés (appareil, chemins) | `camera_validation` |
| Suivi de progression facultatif | Rien n'est enregistré ni écrit si désactivé | `nothing_is_recorded_or_written_when_tracking_is_off` |
| Effacement des données d'intégration | Écrasement par des zéros puis suppression | `erase_all_removes_everything` |

**Limite de l'effacement** : sur SSD, systèmes de fichiers journalisés ou avec copie sur écriture, l'écrasement n'est
pas une garantie d'effacement physique. Pour une suppression sécurisée des données clients, le chiffrement des volumes et la
destruction de clé seront nécessaires (prévu).

## Comptes et authentification locaux (v0.1, testés)

| Garantie | Mécanisme | Test |
|---|---|---|
| Aucun mot de passe en clair | Argon2id (19 Mio, t=2, p=1), sel aléatoire, paramètres stockés avec le hachage | `password_hash_roundtrip_and_no_plaintext` |
| Pas d'énumération de comptes | Mot de passe toujours vérifié ; même réponse pour courriel inconnu et mauvais mot de passe | `login_flow_and_generic_errors` |
| Force brute freinée | 3 essais libres, puis délai exponentiel, verrouillage 15 min après 6 échecs ; le bon mot de passe est refusé pendant le délai | `brute_force_is_throttled_then_locked` |
| 2FA TOTP | RFC 6238 (vecteurs officiels), tolérance ±1 pas, anti-rejeu ; secret **chiffré** (ChaCha20-Poly1305, clé dérivée du mot de passe) | `rfc6238_test_vectors`, `two_factor_enrolment_login_and_replay` |
| Récupération | 10 codes à usage unique stockés hachés ; réinitialise le mot de passe, **désactive le 2FA**, révoque les sessions ; soumise à la limitation | `recovery_code_is_single_use_and_disables_2fa` |
| Sessions | Jeton de 256 bits, seul son SHA-256 est stocké ; inactivité 30 j / durée absolue 90 j ; révocation individuelle ou globale | `token_roundtrip_and_hash_only`, `revocation` |
| Changement de mot de passe | Rechiffre le secret 2FA, révoque **toutes** les sessions | `password_change_reseals_2fa_and_revokes_sessions` |
| Fichier de compte | Écriture atomique, permissions 0600 (Unix) | `store_roundtrip_permissions_and_deletion` |
| Export de données | Sans aucun secret (ni hachage, ni 2FA, ni jeton) | `export_contains_no_secrets` |
| Adresses : rien ne sort sans accord | Le fournisseur n'est **jamais appelé** sans consentement `AddressLookup` | `provider_is_never_called_without_consent` |
| Assistant : permission de lire la photo | Aucune mesure ni proposition sans permission ; rien n'est modifié sans confirmation | `never_looks_at_the_photo_without_permission`, `proposal_preview_confirmation_apply_undo_redo` |
| Synchronisation de la progression | Charge utile refusée sans consentement `ProgressSync` | `state_persists_locally_and_sync_requires_consent` |
| Noms de fichiers à l'exportation | `..`, séparateurs, noms réservés Windows neutralisés ; Unicode conservé ; jamais d'écrasement | `resize_and_unicode_names_and_hostile_names` |

**Limites connues de cette couche locale** : (1) le secret 2FA est protégé par le mot de passe, pas par le coffre de l'OS (prévu) ;
(2) un attaquant ayant un accès **local** au fichier peut tenter une attaque hors ligne sur le hachage (Argon2id la ralentit) ;
(3) la limitation de tentatives locale est contournable en modifiant le fichier : le service en ligne devra imposer la sienne ;
(4) **aucune vérification de courriel, récupération par courriel ni synchronisation** : ces fonctions exigent le service en ligne (ARCHITECTURE.md §7) ;
(5) l'indicateur de robustesse est une heuristique d'aide, pas une garantie ; (6) aucune revue de sécurité externe n'a eu lieu.

## Vérification du courriel et anti-robot (`norelias-server`, testés avec des doubles de fournisseurs)

| Garantie | Mécanisme | Test |
|---|---|---|
| Compte inactif tant que non vérifié | Statut `Pending` : aucune session, même avec le bon mot de passe | `full_journey_register_verify_activate_login` |
| Code non devinable | 6 chiffres uniformes (échantillonnage par rejet), 5 essais par code puis verrouillage, TTL 10 min | `codes_are_six_digits_and_roughly_uniform`, `attempts_are_limited…` |
| Code non récupérable depuis la base | HMAC-SHA256 lié à l'adresse, au but et au secret serveur ; jamais en clair | `code_is_never_stored_and_is_bound…`, `persisted_state_contains_no_plaintext_secrets…` |
| Renvois contrôlés | ≥ 60 s entre deux envois, 5 par heure par adresse, ancien code invalidé | `code_expiry_resend_cooldown_invalidation_and_limits` |
| **Pré-détournement** impossible | Activation = code **et** jeton secret du client d'inscription | `pre_hijacking_is_blocked_by_the_signup_token` |
| Pas d'énumération | Réponses identiques (inscription, récupération, renvoi, connexion) | `no_account_enumeration` |
| CAPTCHA réel, côté serveur, en premier | Jeton vérifié auprès du fournisseur avant toute logique ; nom d'hôte et score contrôlés ; **échec fermé** | `captcha_is_checked_server_side_before_anything_else`, `captcha_provider_outage_fails_closed…`, `real_verifier_fails_closed` |
| Connexions suspectes | CAPTCHA exigé après 2 échecs sur le compte ou 3 depuis l'IP | `suspicious_logins_require_a_real_captcha` |
| Changement d'adresse | Mot de passe + session + code envoyé à la **nouvelle** adresse ; ancienne adresse avertie ; sessions révoquées | `email_change_requires_verification_of_the_new_address` |
| Récupération | CAPTCHA + code par courriel ; **avec 2FA actif, un code de récupération est aussi exigé** ; sessions révoquées ; courriel de sécurité | `password_recovery_journey`, `two_factor_is_optional_and_email_alone_cannot_bypass_it` |
| Abus | Plafonds par IP (inscription 5/h, vérification 30/h, récupération 5/h) | `abuse_limits_per_ip` |
| Erreurs de livraison | Transitoire : reprises avec délai croissant ; définitive : rebond signalé, aucun compte fantôme | `delivery_failures_are_handled` |
| **Rien de simulé en production** | `Server::new` refuse courriel/CAPTCHA simulés, secret faible (< 32 octets ou nul), URL non HTTPS | `nothing_simulated_in_production` |
| Courriels sûrs | Variables échappées, aucune image distante ni pixel de suivi, liens HTTPS, adresses masquées dans les alertes | `all_variables_are_html_escaped`, `responsive_accessible_and_private_by_construction` |
| Conditions | Acceptations obligatoires avec version et date ; marketing distinct, décoché, historique ajouté seulement | `acceptance_of_terms_and_privacy_is_mandatory_and_recorded` |

**Limites** : (1) les fournisseurs de courriel et de CAPTCHA sont des **doubles de test** — aucun test n'a joint un service réel ; les formats d'API sont écrits d'après leur documentation rappelée, non vérifiés ; (2) l'état est un fichier JSON mono-processus, pas une base transactionnelle ; (3) l'adresse IP du client dépend de la configuration du proxy (à fiabiliser) ; (4) la demande de CAPTCHA après échecs révèle faiblement qu'un compte existe (compromis assumé, limité aux comptes connus) ; (5) aucun test d'intrusion ni revue externe ; (6) la file d'envoi est en mémoire : un plantage perd les envois en attente (l'utilisateur redemande un code) ; (7) DNS non publié : SPF/DKIM/DMARC non effectifs tant qu'ils ne sont pas créés par le propriétaire du domaine.

## Protection des données personnelles (Loi 25, LPRPDE, RGPD) — état

Implémenté : minimisation (seuls courriel + mot de passe requis, et seulement pour un compte), consentements par finalité et révocables,
export JSON des données du compte, suppression du compte (écrasement puis suppression), aucun envoi de photos ni de données sans accord.
**Reste à faire (juridique et produit)** : politique de confidentialité rédigée et relue par un juriste, responsable de la protection des renseignements,
registre des incidents, évaluation des facteurs relatifs à la vie privée, préférences de communication, localisation des serveurs, contrats avec les sous-traitants
(fournisseur d'adresses, hébergement de galeries). **Photos de clients : jamais utilisées pour entraîner un modèle sans autorisation spécifique** — aucun entraînement n'existe dans le code ; la règle sera une case dédiée, décochée par défaut, par séance.

## Non implémenté (ne pas considérer comme protégé)

* Coffre sécurisé pour clés API et secrets 2FA (Keychain macOS / Credential Manager Windows) — prévu ; **aucune clé d'API n'est utilisée à ce jour** (fournisseur d'adresses non choisi).
* Validation et isolation du décodage de fichiers hostiles : la CLI utilise la crate `image` (Rust sûr, mais non fuzzée par nous) ; les décodeurs RAW (C/C++) exigeront un processus isolé et du fuzzing.
* Permissions minimales de l'application, signature du code, mises à jour signées.
* Chiffrement des catalogues, galeries privées, mots de passe, liens expirables (module K).
* Journal de consentements infalsifiable : le registre est un JSON local modifiable par l'utilisateur (acceptable pour un usage personnel, pas pour une preuve juridique).
* Audit de licences/vulnérabilités en CI (`cargo-deny`, `cargo-audit`) — à ajouter.

## Règles de développement

1. Toute nouvelle dépendance : licence vérifiée (LICENSES.md), absence de bibliothèque réseau (ou exception écrite + test mis à jour).
2. Tout envoi de données hors de l'ordinateur passe par `ConsentLedger::require` avec une finalité nommée, et est affiché à l'utilisateur.
3. Les fichiers importés sont non fiables ; aucune écriture hors des dossiers choisis.
4. Aucune clé ou secret dans le dépôt, dans le profil ou dans les journaux.
