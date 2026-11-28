# Design system

> **Maquettes** : quatre captures ont été fournies (espace de travail « Développement RAW », inscription en 4 étapes,
> fiche d'aide « Découvrir l'exposition », création de filigrane). §5 les analyse et signale où elles diffèrent des règles du projet.
> Aucune interface graphique n'est encore construite : ce document est le contrat que l'interface devra respecter.

## 1. Couleurs (thème sombre, du brief) et contrastes **calculés** (WCAG 2.x)

| Jeton | Valeur | Usage |
|---|---|---|
| `--bg` | #18191B | Fond principal |
| `--panel` | #222326 | Panneaux |
| `--surface` | #2C2D30 | Surfaces secondaires, champs |
| `--text` | #F1F1F1 | Texte principal |
| `--text-2` | #A4A7AD | Texte secondaire |
| `--accent` | #C8AA7C | Accent champagne |

| Combinaison | Ratio | Verdict |
|---|---|---|
| Texte sur fond | 15,58 | AAA |
| Texte sur panneau | 13,91 | AAA |
| Texte sur surface | 12,19 | AAA |
| Texte secondaire sur fond / panneau / surface | 7,30 / 6,52 / 5,71 | AA (AAA sur fond) |
| Accent sur fond / panneau / surface | 7,96 / 7,11 / 6,23 | AA (AAA sur fond, panneau) |

Le thème clair n'est **pas** défini par le brief : à concevoir et à mesurer avec la même méthode avant adoption.
La couleur d'un constat du conseiller ne doit jamais être le seul porteur de la gravité (icône + libellé : Info / Avertissement / Fort).

## 2. Typographie, icônes, densité

Inter (SIL OFL) ; icônes fines à trait constant, sans emprunt aux icônes d'éditeurs tiers. Densité pilotée par le mode :
débutant (réglages essentiels, explications simples), intermédiaire (standard), expert (complet, aucune explication).

## 3. Composition de l'espace de travail

Barre supérieure (menus natifs, espaces) · panneau gauche contextuel · **photographie centrale** · panneau de réglages à droite ·
film fixe en bas · barre d'état (zoom, dimensions, profil, progression). Panneaux redimensionnables, rétractables, réorganisables.

## 4. Règles issues du module M (testées côté données)

1. Un outil moins prioritaire est **replié** ou **à la demande**, jamais absent : la recherche d'outils (Ctrl/⌘+K) atteint tout.
2. Un outil `Planned` s'affiche grisé avec « Prévu — pas encore disponible », sans curseur ni bouton d'action.
3. Les constats du conseiller montrent la mesure, l'explication, l'aperçu avant/après et deux actions : **Appliquer** / **Ignorer**.
   « Tout corriger automatiquement » n'inclut que les corrections sûres et affiche la liste de ce qui a été appliqué, annulable.
4. Aides désactivées = aucune infobulle, aucun encart, aucune proposition de tutoriel ; l'aide reste atteignable via le menu Aide.
5. Le questionnaire : une question par écran, toujours un bouton « Passer », un résumé final modifiable, la mention
   « Vos réponses restent sur cet ordinateur ».
6. Aucun consentement n'est pré-coché ; un service payant affiche son coût avant la case de consentement.

## 5. Analyse des maquettes fournies et écrans à construire

### 5.1 Ce que les maquettes montrent (à respecter)
* **Barre supérieure** : logo « Norélias Studio Pro — Suite photographique professionnelle tout-en-un », modules Accueil · Bibliothèque · Sélection IA · Développement RAW · Éditeur avancé · Portrait · Studio IA · Composition · Impression · Galeries clients · Exportation ; module actif en champagne ; pastille de synchronisation, notifications, réglages.
* **Gauche** : catalogue (recherche, Toutes les photos, Récemment importées, Modifiées, Favoris, Rejetées, Doublons), projets/séances en arbre, collections intelligentes, mots-clés. **Centre** : photo, avant/après à curseur, notes, étiquettes de couleur, 1:1 / Fit / 100 %, film fixe. **Droite** : onglets Réglages · Masques · Calques · Presets ; histogramme, profil, balance des blancs, tonalité, courbe, HSL ; sections Studio IA, Corrections optiques, Effets. **Bas** : raccourcis de modules et barre d'état (format, appareil, dimensions, ISO, focale, ouverture, vitesse).
* **Inscription** : 4 segments de progression (Compte · Studio · Expérience · Préférences), champs arrondis, bouton « Continuer » clair, indicateur de robustesse à 4 segments.
* **Fiche d'aide** : titre avec icône, trois vignettes sous-exposée / bien exposée / surexposée, « À quoi sert cet outil ? », « Comment l'utiliser ? », « Conseil professionnel », puis actions.
* **Filigrane** : aperçu sur image, case d'activation, nom, style (« Élégant »), opacité (80 %), taille (22 px), grille de position 3×3.

### 5.2 Écarts à corriger avant de bâtir l'interface (règle N-11 : aucune fonction factice)
| Maquette | Problème | Règle appliquée |
|---|---|---|
| Interrupteurs « Studio IA » **activés** (retouche de peau IA, réduction du bruit IA, équilibrage des couleurs IA…), onglets Masques / Calques / Presets, listes de masques (Sujet, Ciel, Personnes…) | Ces outils n'ont **aucun moteur** à ce jour | Affichés **grisés** avec « Prévu — pas encore disponible », sans interrupteur ni curseur actif (`Status::Planned`) |
| Pastille « Synchronisé » | Aucun service de synchronisation n'existe | Masquée en mode local ; n'apparaît qu'avec un service réel **et** le consentement `AccountSync` |
| Compteurs (42 380 photos, 368 images…) et histogramme | Données d'exemple | Les compteurs viennent du catalogue réel ; histogramme : prévu |
| Fiche d'aide : « Options disponibles dans le futur logiciel : Tutoriel vidéo… » | Annonce une vidéo inexistante | « Tutoriel vidéo » **absent** tant qu'aucune vidéo n'existe ; message « Aucun tutoriel vidéo disponible pour le moment » (testé) |
| Fiche d'aide : trois **portraits de personnes différentes** | Le brief exige la **même** photographie avant/après ; droits des visages inconnus | Exemples générés depuis une même image d'entraînement libre de droits (`help::example`) |
| Fiche d'aide : « Ne plus afficher cet explication » | Faute d'accord | « Ne plus afficher pour cet outil » |
| Filigrane : libellé « Ajouter mon filigrane » **barré** | Défaut de rendu probable (texte barré ≠ case cochée) | Libellé normal ; état exprimé par la case |
| Inscription : champ intitulé « Mot de passe fictif pour la démonstration » | Libellé de démonstration | « Mot de passe » ; bouton Afficher/Masquer ajouté |
| Panneau « Corrections optiques » : « Supprimer tla distorsion » | Coquille visible (et le rendu de « Amélioration de netteté » est brouillé) | À corriger dans la maquette finale |
| Photographies de la maquette (mère et fille, portraits) | Droits et consentement inconnus ; rendu probablement généré | **Non reproduites** dans le logiciel ni l'Académie |

### 5.3 Écrans (contrat d'interface — non construits)
1. **Inscription** (4 étapes). *Compte* : prénom, nom, courriel, mot de passe, confirmation, œil afficher/masquer, barre de robustesse dynamique (clés `password.strength.*`), message de correspondance, lien « Utiliser sans compte (mode local) ». *Studio* : case « Je n'ai pas encore de studio commercial », nom, slogan, logo, adresse avec suggestions (si consentement) ou saisie manuelle, étiquettes de région/code postal selon le pays, téléphone avec indicatif, site, réseaux, informations fiscales. *Expérience* : niveau (3), spécialités. *Préférences* : langue, fuseau, devise, unités. Chaque étape a **Retour** et **Passer** (sauf Compte, qui n'a que « sans compte »). Les erreurs s'affichent sous le champ avec une icône et un libellé (pas seulement la couleur).
2. **Connexion** : courriel, mot de passe, code 2FA si requis, message d'attente « Réessayez dans N secondes » (`auth.locked`), erreur générique unique (pas d'énumération), lien Récupération.
3. **Récupération du compte** : code de récupération + nouveau mot de passe (version locale) ; version courriel marquée « service en ligne requis » tant qu'il n'existe pas. Rappel : la récupération **désactive le 2FA** et révoque les sessions.
4. **Configuration initiale** : récapitulatif modifiable des réponses (module M) et des consentements, tous décochés par défaut ; affichage **une seule fois** des codes de récupération avec « Copier / Télécharger / J'ai noté mes codes ».
5. **Mon Studio** : identité, logo, copyright, coordonnées, fiscalité, devise, unités ; aperçu de la façon dont le nom alimente le filigrane.
6. **Préférences** : niveau, aides (infobulles, accompagnement, Réinitialiser les tutoriels), langue/apparence, raccourcis, confidentialité (une ligne par finalité : état, fournisseur, coût, Retirer), sécurité (2FA, appareils et sessions avec Révoquer, changer le mot de passe), données (Exporter mes données, Supprimer mon compte avec confirmation).
7. **Centre d'aide** : recherche, fiches d'outils, état « vidéo non disponible », assistant (avec case « Autoriser l'assistant à voir ma photo », désactivée par défaut, et mention « assistant à règles »).
8. **Académie** : parcours par niveau et spécialité, exercices, progression facultative, bandeau « contenu prévu » pour ce qui n'existe pas.
9. **Création de filigranes** : aperçu temps réel sur la photo ouverte, case « Ajouter mon filigrane », source du texte (nom du studio / photographe / personnalisé / copyright), style, couleur, taille, opacité, rotation, grille de position 3×3 + position libre, marges, ombre, contour, répétition, réglages portrait/paysage, modèles enregistrés, **import de logo PNG** (SVG « prévu »). Dans l'export : « Ajouter mon filigrane » + « Exporter aussi une version sans filigrane » ; copyright invisible dans une section distincte.

### 5.4 Règles transversales
* **i18n** : aucun texte dans les composants, tout par clé du catalogue ; les écrans doivent supporter un texte 40 % plus long et un miroir **droite-à-gauche** (propriétés logiques CSS : `margin-inline`, `inset-inline`) ; formats via `norelias-i18n`.
* **Accessibilité** : chaque commande a une infobulle accessible (étiquette ARIA), navigation clavier complète, contrastes du §1.
* **Photo au centre** : les écrans de compte et d'apprentissage sont des feuilles/volets, jamais un tableau de bord qui remplace l'espace photo.
