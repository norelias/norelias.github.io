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

## Non implémenté (ne pas considérer comme protégé)

* Coffre sécurisé pour clés API (Keychain macOS / Credential Manager Windows) — prévu.
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
