# Migration depuis d'autres logiciels

Ce document décrit **ce qui fonctionne réellement aujourd'hui** (v0.1) et ce qui est prévu. Aucune ressource propriétaire
(icônes, textes, dispositions, profils d'appareils, presets d'éditeurs) n'est incluse ni nécessaire.

> **Fidélité.** Les algorithmes des autres logiciels sont propriétaires. Un réglage converti est une *approximation
> visuelle* : vérifiez toujours le résultat. Aucun rendu pixel à pixel identique n'est promis.

## 1. Raccourcis clavier (testé, couverture partielle)

Choix dans Préférences → Raccourcis ; proposé automatiquement d'après les logiciels cités au questionnaire.
Les raccourcis sont des conventions d'usage ; une commande non redéfinie par le profil retombe sur la disposition Norélias.

| Commande | Norélias | Type Photoshop | Type Lightroom | Type Capture One |
|---|---|---|---|---|
| Annuler | Ctrl/⌘+Z | Ctrl/⌘+Alt+Z | Ctrl/⌘+Z | Ctrl/⌘+Z |
| Rétablir | Ctrl/⌘+Maj+Z | Ctrl/⌘+Maj+Z | Ctrl/⌘+Maj+Z | Ctrl/⌘+Maj+Z |
| Recadrage | C | C | R | C |
| Exporter | Ctrl/⌘+Maj+E | Ctrl/⌘+Alt+Maj+S | Ctrl/⌘+Maj+E | Ctrl/⌘+Maj+E |
| Grille / Loupe / Développement | G / E / D | (base) | G / E / D | (base) |
| Notes 0–5, Retenir P, Rejeter X | idem | (base) | idem | idem |

Toutes les touches restent **personnalisables** ; une combinaison déjà utilisée est refusée (détection de conflits testée).
Ces profils n'ont **pas** été validés par d'anciens utilisateurs : retours bienvenus.

## 2. Presets XMP (testé, partiel)

Les attributs `crs:*` d'un fichier `.xmp` de preset sont lus. Chaque réglage est soit converti en opération réelle,
soit listé dans le **rapport de migration** avec la raison.

| Réglage source | Converti en | Remarque |
|---|---|---|
| Exposure / Exposure2012 | Exposition (IL) | Valeur directe |
| Contrast / Contrast2012 (−100…100) | Contraste | ×0,5 : plage plus douce |
| Saturation (−100…100) | Saturation | 1 + v/100 |
| Highlights2012 négatif | Hautes lumières | Positif : non converti (pas d'équivalent) |
| Shadows2012 positif | Ombres | Négatif : non converti |
| Sharpness (0…150) | Netteté | v/150 |
| LuminanceSmoothing (0…100) | Réduction du bruit | v/100 |
| Temperature, Tint | **non converti** | Dépend du profil d'appareil et du RAW |
| Vibrance, Clarity, Texture, Dehaze, Whites, Blacks | **non converti** | Outil pas encore intégré |
| Courbes, HSL, grain, vignettage, correction d'objectif | **non converti** | Outils prévus |

Les attributs de métadonnées (version, UUID, restrictions d'appareil…) sont ignorés.

## 3. Catalogues (prévu)

* Reconnaissance par signature : catalogue Lightroom (`.lrcat`, base SQLite), catalogue Capture One, PSD — **testée**.
* **Import non implémenté.** Plan : lecture seule du catalogue source (jamais modifié), copie des chemins, notes, mots-clés,
  drapeaux, étiquettes de couleur et métadonnées XMP quand elles sont présentes ; rapport des éléments non migrés ;
  réglages de développement : uniquement ceux du tableau §2.
* En attendant : exportez les métadonnées en fichiers `.xmp` « sidecar » depuis l'ancien logiciel.

## 4. PSD (prévu)

Import/export PSD limités aux calques pixel, masques, opacité et modes de fusion courants ; objets dynamiques, calques de
réglage, effets de calque et texte éditable seront documentés comme incompatibles tant qu'ils ne sont pas supportés.
