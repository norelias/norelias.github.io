# Site Norélias Photography — norelias.ca

## Ce qu'il y a dans le dossier

- `index.html` : l'accueil
- `a-propos/`, `prestations/`, `portfolio/`, `tarifs/`, `reservation/`, `contact/`, `politique-reservation/`, `confidentialite/` : les autres pages
- `images/` : les photos. **Ce sont des photos temporaires.** Remplacez-les par les vôtres en gardant le même nom de fichier (ex. `accueil-fond.jpg` pour la grande photo d'accueil, `souvenirs-fond.jpg` pour le bébé au lapin).
- `config.js` : **le seul fichier à modifier** pour les liens de paiement Stripe et l'envoi des courriels
- `style.src.css` : l'apparence (à modifier ici), `style.css` est généré à partir de ce fichier
- `script.js` : le fonctionnement (réservation, filtres, menu)
- `CNAME` : contient `norelias.ca`, pour GitHub Pages
- `outils/construire.py` : sert à régénérer les pages si on modifie les textes

## Mettre le site en ligne (GitHub Pages, comme academiesere.ca)

1. Sur GitHub, créez un nouveau dépôt public nommé `norelias`.
2. Cliquez sur **Add file > Upload files** et glissez tout le contenu de ce dossier (pas le dossier lui-même). Cliquez sur **Commit changes**.
3. Allez dans **Settings > Pages**. Sous **Branch**, choisissez `main` et `/ (root)`, puis **Save**.
4. Dans le même écran, sous **Custom domain**, écrivez `norelias.ca` et cliquez sur **Save**.
5. Chez le fournisseur où vous avez acheté norelias.ca, ajoutez ces enregistrements DNS :
   - 4 enregistrements **A** pour `@` : `185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153`
   - 1 enregistrement **CNAME** pour `www` : `serenaissancedoula-hue.github.io`
6. Attendez de quelques minutes à quelques heures. Puis, dans **Settings > Pages**, cochez **Enforce HTTPS**.

## Paiements Stripe

Dans Stripe, créez 4 **liens de paiement** (Payment Links) :

| Lien | Montant |
|---|---|
| Acompte forfait Essentiel | 100 $ |
| Acompte forfait Signature | 150 $ |
| Acompte forfait Complète | 225 $ |
| Mini-séance de Noël (paiement complet) | 95 $ |

Collez chaque lien dans `config.js`. Tant qu'un lien est vide, la cliente est dirigée vers le virement Interac.

## Courriels de réservation

Sans réglage, la demande s'ouvre dans l'application courriel de la cliente, adressée à bonjour@norelias.ca.
Pour un envoi automatique, créez des modèles dans EmailJS (comme pour academiesere.ca) et remplissez la section `emailjs` de `config.js`.
