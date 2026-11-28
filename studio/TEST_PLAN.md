# Plan de tests

Pyramide : unitaires → scénarios sur images → bout en bout (binaire réel) → visuels/matériels (prévus).
Lancer : `cargo test --workspace` (≈ 30 s, dont Argon2 à coût réduit) ; mesures : `cargo test --release -p norelias-advisor --test perf -- --ignored --nocapture`.

## 1. Résultats actuels (Linux x86-64, Rust 1.97, 4 cœurs)

**285 tests, 0 échec** ; 2 tests de performance (ignorés par défaut).

| Suite | Tests | Ce qui est vérifié |
|---|---|---|
| `norelias-core` | 18 | Aller-retour sRGB exact sur 256 valeurs ; chaque opération (identité, valeurs connues) ; source jamais modifiée ; sérialisation ; débruitage préserve les contours |
| `norelias-advisor` (scénarios) | 13 | 9 défauts injectés (sous/sur-exposition + écrêtage, dominante chaude, teinte verte, bruit, flou, bougé, horizon incliné −4°/2°/5°, sujet centré) détectés **puis corrigés** et ré-analysés ; aucun faux positif sur la scène de référence ; aucune correction inventée pour le bougé ; corrections créatives jamais automatiques ; textes FR/EN |
| `norelias-onboarding` | 48 | Profil, migrations, corruption, version future, écriture atomique, effacement ; consentements ; registre d'outils ; **aucun outil supprimé** (3 modes × 14 spécialités × aides on/off) ; raccourcis sans conflit ; import XMP ; questionnaire (réponses complètes, passer, entrées invalides, fermeture, révocation) |
| `norelias-academy` | 37 | Leçons FR/EN complètes et liées aux outils réels ; exercices (inactivité = échec, correction du conseiller = réussite, mauvaises réponses = échec) ; recherche ; infobulles ; suivi facultatif ; aide contextuelle ; aides désactivées |
| `norelias-cli` (e2e) | 11 | Binaire réel sur vrais PNG : analyse, correction automatique/choisie, **original intact**, refus d'écraser, questionnaire par stdin, profil local, workspaces, recherche, garde anti-réseau |

| `norelias-i18n` | 16 | Parité FR/EN des clés et des paramètres, repli de langue, nombres/monnaies/dates par langue, années bissextiles, fuseaux, unités, taxes (composées, TTC→HT) |
| `norelias-account` | 50 | Argon2id et chiffrement (altération détectée), **vecteurs RFC 6238**, limitation, sessions, mots de passe, courriels/URL, **téléphones E.164**, codes postaux de 15 pays, porte de consentement des adresses, inscription par étapes, compte (connexion, 2FA, récupération, export sans secret, permissions 0600, suppression) |
| `norelias-studio` | 27 | Filigrane : rendu réel de texte Unicode et de logo transparent, positions, marges, opacité 0 = invisible, rotation, ombre/contour/répétition, portrait/paysage, aperçu fidèle ; export : **case décochée = octet pour octet identique**, double version, refus d'écraser sans écriture partielle, copyright séparé, JPEG + XMP, noms hostiles, lot |
| `norelias-server` | 54 | Codes (uniformité, TTL, essais, renvoi, HMAC), conditions (versions, historique marketing), courriels (classification HTTP, requêtes Postmark/Resend, reprises, rebonds), CAPTCHA (3 fournisseurs, échec fermé), SPF/DKIM/DMARC, 13 modèles ; **20 parcours** : inscription→activation→connexion, pré-détournement, anti-énumération, changement d'adresse, récupération, 2FA, limites par IP, suppression, persistance, garde de production, traductions |
| `norelias-assistant` | 9 | Intentions FR/EN, outils réels seulement, **aucune lecture de la photo sans permission**, aperçu, confirmation, application, annulation/rétablissement, no-op signalé, demande contredite par la mesure |

Mesures réelles (24 Mpx, mono-thread) : voir ARCHITECTURE.md §5 et §8.

## 2. Limites des tests actuels (à lire avant de se fier aux résultats)

* **Données synthétiques uniquement.** Les seuils du conseiller ne sont pas calibrés sur de vraies photographies ; faux positifs
  attendus (contre-jour voulu, scènes sans contours, horizons naturellement inclinés). Dit à l'utilisateur dans chaque explication.
* **Linux uniquement.** Aucun test Windows/macOS.
* **Aucune interface graphique** donc aucun test visuel ni d'accessibilité.
* L'e2e utilise un dossier de données temporaire ; aucun test d'installation.

## 2 ter. Limites propres à la vérification du courriel

* **Aucun test n'a envoyé de courriel réel ni vérifié un vrai CAPTCHA** : les fournisseurs sont des doubles ; les JSON de réponse des CAPTCHA sont des exemples écrits d'après la documentation rappelée.
* Aucun test dans de vrais clients de messagerie (Gmail, Outlook, Apple Mail), ni de délivrabilité, ni de mode sombre réel.
* DNS : validation de **syntaxe** seulement ; aucune interrogation du DNS.
* Pas de test de charge, de concurrence (état mono-processus) ni d'intrusion.

**Tests d'acceptation à exécuter avec les vrais services** : ARCHITECTURE.md §9.4.

## 2 bis. Limites propres à l'extension

* **Aucun fournisseur d'adresses réel** : l'autocomplétion est testée avec un fournisseur simulé défini **dans les tests** ; aucun test ne prouve la qualité de suggestions réelles.
* **Aucun service en ligne** : vérification du courriel, récupération par courriel, synchronisation, révocation distante : non testables.
* Argon2 est exécuté à coût réduit dans les tests ; le coût de production (19 Mio) n'est pas chronométré sur machine cible.
* Les textes des fiches et leçons ne sont **pas relus** par des photographes ; l'assistant n'a pas été évalué avec de vrais utilisateurs.
* Rendu des polices : testé par mesure de pixels (position, présence), pas par comparaison visuelle ; Inter et Liberation Serif uniquement.
* Conformité juridique (Loi 25, LPRPDE, RGPD) : non testée, revue juridique requise.

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

## 4 bis. Plan de l'extension (couverture)

| Exigence | Tests |
|---|---|
| N-A01…N-A07 | `signup::tests`, `address::tests` |
| N-A09, N-A10 | `phone::tests`, `validate::tests` |
| N-A11…N-A15 | `record::tests`, `totp::tests`, `limiter::tests`, `session::tests`, `crypto::tests` |
| N-B01, N-C01…N-C12 | `help::tests` (11), `workspace::tests` |
| N-D01…N-D05 | `lessons::tests`, `search::tests`, `exercises::tests` |
| N-E01…N-E06 | `norelias-assistant` (9), e2e `assistant_needs_permission_and_never_applies` |
| N-F01…N-F14 | `watermark::tests` (14), `export::tests` (9), `templates::tests`, e2e `export_watermark_checkbox_and_dual_versions` |
| N-G01…N-G10 | `norelias-i18n` (16) |
| N-H01…N-H07 | `privacy::tests`, `record::tests::export_contains_no_secrets`, `help::tests::state_persists…` |

**À ajouter (extension)** : tests de bout en bout de l'interface d'inscription (clavier, lecteur d'écran, texte long, droite-à-gauche) ; test d'intrusion de la couche d'authentification ; test d'intégration avec le fournisseur d'adresses choisi (y compris hors connexion et quotas) ; test de charge du service en ligne ; audit d'accessibilité WCAG des 9 écrans ; revue juridique.

## 4 ter. Plan de la vérification du courriel (couverture)

| Exigences | Tests |
|---|---|
| V-01…V-06 | `code::tests` (6), `journeys::code_expiry_resend…`, `address_can_be_corrected…` |
| V-07, V-08, V-12 | `journeys::full_journey…`, `pre_hijacking…`, `email_change…`, `password_recovery_journey`, `two_factor…` |
| V-09, V-16 | `persisted_state…`, `nothing_simulated_in_production` |
| V-10, V-11 | `mail::tests` (6), `delivery_failures_are_handled` |
| V-13 | `dns::tests` (5), e2e `mail_templates_preview_and_dns_plan` |
| V-14 | `templates::tests` (8), e2e `mail_templates_preview_and_dns_plan` |
| V-17, V-18 | `captcha::tests` (5), `captcha_*`, `suspicious_logins…` |
| V-19…V-21 | `terms::tests` (4), `acceptance_of_terms…`, `marketing_can_be_withdrawn…` |
| V-22 | `every_error_key_has_a_translation…` |

## 5. À ajouter

Jeu de photographies autorisées (RAW + JPEG) avec vérités terrain pour calibrer le conseiller ; tests visuels de non-régression sur
images de référence ; CI Windows/macOS/Linux ; fuzzing des décodeurs ; tests utilisateurs de l'onboarding (taux de complétion,
compréhension des modes, profils de raccourcis).
