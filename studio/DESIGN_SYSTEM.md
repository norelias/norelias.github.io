# Design system

> **Maquettes** : le brief mentionne des maquettes visuelles de référence ; **aucune maquette n'a été fournie à cette
> session**. Ce document applique la palette textuelle du brief ; les écrans seront alignés sur les maquettes dès qu'elles seront transmises.

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
