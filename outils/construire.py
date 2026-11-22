# Génère les pages HTML du site Norélias (fidèle à la maquette).
# Utilisation : python3 outils/construire.py  (depuis le dossier du site)
import os, re

RACINE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SITE = 'https://norelias.ca'

FB = '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 2a10 10 0 0 0-1.6 19.9v-7H7.9V12h2.5V9.8c0-2.5 1.5-3.9 3.8-3.9 1.1 0 2.2.2 2.2.2v2.5h-1.3c-1.2 0-1.6.8-1.6 1.6V12h2.8l-.4 2.9h-2.4v7A10 10 0 0 0 12 2z"/></svg>'
IG = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" aria-hidden="true"><rect x="3" y="3" width="18" height="18" rx="5"/><circle cx="12" cy="12" r="4"/><circle cx="17.5" cy="6.5" r="1.2" fill="currentColor" stroke="none"/></svg>'
TT = '<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M16.6 5.8A4.3 4.3 0 0 1 15.5 3h-3.1v12.4a2.6 2.6 0 1 1-1.8-2.5V9.7a5.7 5.7 0 1 0 4.9 5.7V9a7.3 7.3 0 0 0 4.3 1.4V7.3a4.3 4.3 0 0 1-3.2-1.5z"/></svg>'
COEUR_SVG = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M12 20.5S3.5 14.8 3.5 8.9A4.4 4.4 0 0 1 12 6.8a4.4 4.4 0 0 1 8.5 2.1c0 5.9-8.5 11.6-8.5 11.6z"/></svg>'
COEUR = f'<span class="coeur">{COEUR_SVG}</span>'
FLECHE = '<span class="fleche" aria-hidden="true">→</span>'
FEUILLE_SVG = '''<svg class="feuille" viewBox="0 0 160 200" fill="none" stroke="#3a2a24" stroke-width="1.3" aria-hidden="true"><path d="M60 196 C70 150 72 110 110 60 C125 42 140 30 155 24"/><path d="M78 140c-18-6-30-20-32-38 16 4 28 18 32 38z"/><path d="M84 124c4-18 18-30 36-32-4 16-18 28-36 32z"/><path d="M92 104c-16-8-24-22-24-38 14 6 22 20 24 38z"/><path d="M104 82c6-16 20-24 36-24-6 14-20 22-36 24z"/><path d="M70 160c-16-2-28-12-34-26 16 0 28 10 34 26z"/><path d="M100 50c4 0 8 4 8 8"/><path d="M118 40c-6-14-2-28 8-36 4 14 0 28-8 36z"/></svg>'''
DIVISEUR = f'<div class="separateur" aria-hidden="true">{COEUR_SVG}</div>'

SEANCES = [  # nom, code, image carte, description
    ('Grossesse', 'grossesse', 'grossesse-champ.jpg', "Entre 30 et 36 semaines, en nature à l'heure dorée ou à la maison."),
    ('Accouchement', 'accouchement', 'accouchement.jpg', 'Les premières minutes de votre rencontre, avec douceur et discrétion.'),
    ('Nouveau-né', 'nouveau-ne', 'nouveau-ne.jpg', 'Dans les 5 à 14 premiers jours, au rythme de bébé. Ma spécialité.'),
    ('Famille', 'famille', 'famille-2.jpg', 'Des moments vrais en nature ou à la maison, avec toute la famille.'),
    ('Couple', 'couple', 'couple.jpg', 'Une séance complice pour célébrer votre amour.'),
    ('Fêtes', 'fetes', 'fetes.jpg', 'Anniversaires, gâteau, baptême : les belles occasions à souligner.'),
    ('Saisons', 'saisons', 'saisons.jpg', "Automne, hiver, printemps : des séances au fil des saisons."),
    ('Noël', 'noel', 'noel.jpg', 'Mini-séances de 20 minutes dans un décor de Noël, en décembre.'),
]
UNIVERS = ['grossesse', 'accouchement', 'nouveau-ne', 'famille', 'couple', 'noel', 'fetes']
UNIV_IMG = {'noel': 'noel-2.jpg', 'grossesse': 'grossesse.jpg'}

def reseaux():
    return ('<div class="reseaux">'
            f'<a href="https://www.facebook.com/" aria-label="Facebook">{FB}</a>'
            f'<a href="https://www.instagram.com/" aria-label="Instagram">{IG}</a>'
            f'<a href="https://www.tiktok.com/" aria-label="TikTok">{TT}</a></div>')

def page(chemin, titre, description, contenu, actif=''):
    r = '/' if chemin == '404.html' else '../' * chemin.count('/')
    sous = ''.join(f'<li><a href="{r}prestations/#{c}">{n}</a></li>' for n, c, _, _ in SEANCES)
    def lien(nom, url):
        cur = ' aria-current="page"' if nom == actif else ''
        return f'<a href="{r}{url}"{cur}>{nom}</a>'
    menu = (f'<li>{lien("Accueil", "")}</li><li>{lien("À propos", "a-propos/")}</li>'
            f'<li>{lien("Prestations", "prestations/").replace("</a>", " <span aria-hidden=\"true\">⌄</span></a>")}<ul class="sous-menu">{sous}</ul></li>'
            f'<li>{lien("Portfolio", "portfolio/")}</li><li>{lien("Tarifs", "tarifs/")}</li><li>{lien("Contact", "contact/")}</li>')
    url_page = SITE + '/' + ('' if chemin == 'index.html' else chemin.replace('index.html', ''))
    html = f'''<!DOCTYPE html>
<html lang="fr-CA">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{titre}</title>
<meta name="description" content="{description}">
<link rel="canonical" href="{url_page}">
<meta property="og:title" content="{titre}">
<meta property="og:description" content="{description}">
<meta property="og:image" content="{SITE}/images/accueil-fond.jpg">
<meta property="og:type" content="website">
<meta name="theme-color" content="#3d2622">
<link rel="icon" href="{r}favicon.svg" type="image/svg+xml">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Great+Vibes&family=Sacramento&family=Cormorant+Garamond:ital,wght@0,500;0,600;1,500&family=Nunito:wght@400;600;700;800&display=swap" rel="stylesheet">
<link rel="stylesheet" href="{r}style.css">
</head>
<body>
<a href="#contenu" class="sr-only">Aller au contenu</a>
<header class="entete">
  <div class="barre">
    <a href="{r}" class="logo"><img src="{r}images/logo.png" alt="Norélias Photography" width="246" height="62"></a>
    <nav class="nav" aria-label="Menu principal"><ul class="menu">{menu}</ul></nav>
    {reseaux()}
    <button class="burger" aria-label="Ouvrir le menu" aria-expanded="false"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 6h18M3 12h18M3 18h18"/></svg></button>
  </div>
</header>
<main id="contenu">
{contenu.replace('{r}', r)}
</main>
<footer class="pied">
  <div class="barre">
    <a class="marque" href="{r}">Norélias</a>
    <nav aria-label="Liens du pied de page"><a href="{r}">Accueil</a><span>·</span><a href="{r}prestations/">Prestations</a><span>·</span><a href="{r}tarifs/">Tarifs</a><span>·</span><a href="{r}contact/">Contact</a><span>·</span><a href="{r}politique-reservation/">Politique de réservation</a><span>·</span><a href="{r}confidentialite/">Politique de confidentialité</a></nav>
    <div class="coord"><a href="tel:+15067870667">506-787-0667</a> · <a href="mailto:bonjour@norelias.ca">bonjour@norelias.ca</a></div>
  </div>
</footer>
<script src="{r}config.js"></script>
<script src="{r}script.js"></script>
</body>
</html>
'''
    dest = os.path.join(RACINE, chemin)
    os.makedirs(os.path.dirname(dest) or RACINE, exist_ok=True)
    with open(dest, 'w', encoding='utf-8') as f:
        f.write(html)

def titre_page(h1, sous=''):
    return f'<div class="titre-page"><h1>{h1} {COEUR}</h1>' + (f'<p>{sous}</p>' if sous else '') + '</div>'

# ---------------- Accueil ----------------
def accueil():
    nom = {c: n for n, c, _, _ in SEANCES}
    img = {c: i for _, c, i, _ in SEANCES}
    cartes = ''
    for c in UNIVERS:
        promo = '<span class="etiquette-promo">PROMO 95 $</span>' if c == 'noel' else ''
        cartes += f'<a class="carte-univers" href="{{r}}prestations/#{c}"><span class="photo">{promo}<img src="{{r}}images/{UNIV_IMG.get(c, img[c])}" alt="Séance {nom[c].lower()}" loading="lazy"></span><span class="nom">{nom[c]}</span></a>'
    contenu = f'''
<section class="heros">
  <div class="interieur">
    <div class="heros-texte">
      {FEUILLE_SVG}
      <h1><span class="nom">Norélias</span><span class="metier">PHOTOGRAPHY</span></h1>
      {DIVISEUR}
      <p class="accroche">Immortaliser les moments<br>les plus précieux de votre vie</p>
      <a class="bouton" href="{{r}}reservation/">Réserver ma séance {FLECHE}</a>
    </div>
  </div>
</section>
<section class="bande-noel dechire-haut" style="--fond-dechire:#5c2f2b" aria-labelledby="titre-noel">
  <div class="contenu">
    <img src="{{r}}images/noel.jpg" alt="Petite fille avec une tuque de Noël devant des lumières">
    <div class="texte">
      <div>
        <span class="script">Magie de Noël</span>
        <h2 id="titre-noel">Mini-séances de Noël</h2>
        <div class="details"><span>20 minutes</span><span>10 photos numériques</span><span>95 $</span></div>
        <p class="petit">Places limitées · Dates en décembre</p>
      </div>
      <a class="bouton blanc" href="{{r}}reservation/?seance=noel">Réserver ma place →</a>
    </div>
  </div>
</section>
<section class="souvenirs dechire-bas" style="--fond-dechire:#f6ebe4" role="img" aria-label="Nouveau-né endormi avec un lapin en peluche">
  <div class="interieur">
    <div class="texte">
      <h2><span class="script">Des souvenirs<br>authentiques</span><span class="et">&amp; remplis d'émotions</span></h2>
      {DIVISEUR}
      <p>Grossesse, accouchement, nouveau-né, famille, couple, fêtes ou saisons… Je capture vos histoires avec douceur, naturel et authenticité.</p>
      <a class="bouton" href="{{r}}prestations/">Découvrir mes prestations {FLECHE}</a>
    </div>
  </div>
</section>
<section class="univers">
  <div class="interieur">
    <h2 class="titre-section">Mes univers <em>photo</em>graphiques {COEUR}</h2>
    <div class="grille-univers">{cartes}</div>
  </div>
</section>
<section class="engagements">
  <div class="grille">
    <div class="engagement"><svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="5" y="14" width="38" height="26" rx="4"/><circle cx="24" cy="27" r="8"/><circle cx="24" cy="27" r="4"/><path d="M17 14l3-5h8l3 5M37 19h1"/></svg><span>Séances personnalisées<br>selon vos besoins</span></div>
    <div class="engagement"><svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M24 41S7 31 7 18a9 9 0 0 1 17-4 9 9 0 0 1 17 4c0 13-17 23-17 23z"/></svg><span>Approche humaine<br>et bienveillante</span></div>
    <div class="engagement"><svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M24 40c-6-6-6-16 0-26 6 10 6 20 0 26zM24 40c-8-2-14-9-16-18 8 1 14 6 16 18zM24 40c8-2 14-9 16-18-8 1-14 6-16 18zM24 40c-10 0-17-4-20-10 7-1 14 2 20 10zM24 40c10 0 17-4 20-10-7-1-14 2-20 10z"/></svg><span>Souvenirs intemporels<br>de qualité professionnelle</span></div>
  </div>
</section>'''
    page('index.html', 'Norélias Photography · Photographe nouveau-nés, grossesse et familles à Campbellton',
         "Photographe de nouveau-nés, de grossesse et de familles à Campbellton, N.-B. Séances douces et naturelles, photos numériques en galerie privée. Mini-séances de Noël à 95 $.",
         contenu, 'Accueil')

# ---------------- Prestations ----------------
def prestations():
    cartes = ''
    for nom, code, img, desc in SEANCES:
        promo = '<span class="etiquette-promo">PROMO 95 $</span>' if code == 'noel' else ''
        lien = '?seance=noel' if code == 'noel' else f'?seance={code}'
        cartes += f'''<a class="carte-presta" id="{code}" href="{{r}}reservation/{lien}" title="{desc}"><div class="photo">{promo}<img src="{{r}}images/{img}" alt="Séance {nom.lower()}" loading="lazy"></div><div class="nom">{nom}</div></a>'''
    contenu = f'''<section class="page"><div class="interieur">
{titre_page('Mes prestations', 'Des séances photo pour toutes les étapes<br>importantes de votre vie.')}
<div class="grille-prestations">{cartes}</div>
</div><img class="deco" src="{{r}}images/feuille-1.png" alt="" style="right:calc(50% - 360 * var(--u));bottom:calc(30 * var(--u));width:calc(170 * var(--u))"></section>'''
    page('prestations/index.html', 'Prestations · Norélias Photography', "Séances grossesse, accouchement, nouveau-né, famille, couple, fêtes, saisons et mini-séances de Noël à Campbellton, N.-B.", contenu, 'Prestations')

# ---------------- Portfolio ----------------
def portfolio():
    photos = [('grossesse-champ.jpg', 'grossesse', 'Femme enceinte dans un champ au coucher du soleil'),
              ('nouveau-ne-bonnet.jpg', 'nouveau-ne', 'Nouveau-né endormi avec un bonnet'),
              ('naissance-nb.jpg', 'accouchement', 'Naissance en noir et blanc'),
              ('famille.jpg', 'famille', 'Famille qui marche en nature'),
              ('couple-2.jpg', 'couple', 'Couple au coucher du soleil'),
              ('noel.jpg', 'noel', 'Petite fille avec une tuque de Noël'),
              ('neige.jpg', 'saisons', 'Forêt enneigée en hiver'),
              ('petits-pieds.jpg', 'nouveau-ne', 'Petits pieds de nouveau-né'),
              ('ventre-fleurs.jpg', 'grossesse', 'Ventre de grossesse avec des fleurs')]
    figs = ''.join(f'<figure data-cat="{c}"><img src="{{r}}images/{f}" alt="{a}" loading="lazy"></figure>' for f, c, a in photos)
    cats = [('tous', 'Tous'), ('grossesse', 'Grossesse'), ('accouchement', 'Accouchement'), ('nouveau-ne', 'Nouveau-né'), ('famille', 'Famille'), ('couple', 'Couple'), ('fetes', 'Fêtes'), ('saisons', 'Saisons'), ('noel', 'Noël')]
    boutons = ''.join(f'<button class="filtre" type="button" data-cat="{c}" aria-pressed="{"true" if c == "tous" else "false"}">{n}</button>' for c, n in cats)
    contenu = f'''<section class="page"><div class="interieur">
{titre_page('<span class="script">Portfolio</span>', 'Découvrez une sélection de mes plus belles histoires.')}
<div class="filtres">{boutons}</div><div class="galerie">{figs}</div>
</div></section>'''
    page('portfolio/index.html', 'Portfolio · Norélias Photography', "Photos de grossesse, nouveau-nés, familles, couples et séances de Noël par Norélias Photography, Campbellton.", contenu, 'Portfolio')

# ---------------- À propos ----------------
def apropos():
    contenu = f'''<section class="page"><div class="interieur">
{titre_page('À propos de moi')}
<div class="apropos">
  <img src="{{r}}images/sabrina.jpg" alt="Sabrina, photographe de Norélias">
  <div>
    <p>Je suis Sabrina, photographe passionnée par les histoires vraies, les émotions sincères et les moments uniques de la vie.</p>
    <p>Mon approche est douce, respectueuse et humaine, afin de créer des souvenirs authentiques, remplis de chaleur et d'émotion.</p>
    <a class="bouton" href="{{r}}contact/">En savoir plus {FLECHE}</a>
  </div>
</div>
</div><img class="deco" src="{{r}}images/feuille-2.png" alt="" style="right:calc(50% - 360 * var(--u));bottom:calc(30 * var(--u));width:calc(150 * var(--u))"></section>'''
    page('a-propos/index.html', 'À propos · Sabrina · Norélias Photography', "Je suis Sabrina, photographe et doula à Campbellton. Une approche douce, respectueuse et humaine.", contenu, 'À propos')

# ---------------- Tarifs ----------------
def tarifs():
    contenu = f'''<section class="page"><div class="interieur">
{titre_page('Tarifs &amp; Forfaits', 'Des formules simples, flexibles et adaptées à vos besoins.')}
<div class="grille-tarifs">
  <a class="tarif" href="{{r}}reservation/?forfait=essentiel" style="text-decoration:none;color:inherit"><div class="tete"><div class="montant">200 $</div><h3>Essentiel</h3></div><ul><li>Séance de 30 minutes</li><li>10 photos retouchées</li><li>Galerie privée en ligne</li></ul></a>
  <a class="tarif vedette" href="{{r}}reservation/?forfait=signature" style="text-decoration:none;color:inherit"><div class="tete"><div class="montant">300 $</div><h3>Signature</h3></div><ul><li>Séance de 1 heure</li><li>25 photos retouchées</li><li>Galerie privée en ligne</li><li>Impression 8 x 10</li></ul></a>
  <a class="tarif" href="{{r}}reservation/?forfait=complete" style="text-decoration:none;color:inherit"><div class="tete"><div class="montant">450 $</div><h3>Complète</h3></div><ul><li>Séance prolongée</li><li>Toutes les photos retouchées</li><li>Galerie privée en ligne</li><li>Crédit d'impression</li></ul></a>
</div>
<div class="extras">
  <a class="extra noel" href="{{r}}reservation/?seance=noel"><div><h3>Mini-séance de Noël</h3><p>20 min · 10 photos numériques</p></div><span class="montant">95 $</span></a>
  <a class="extra perso" href="mailto:bonjour@norelias.ca?subject=Forfait%20personnalis%C3%A9"><div><h3>Forfait personnalisé</h3><p>Sur demande : <strong style="color:var(--rose-fonce)">bonjour@norelias.ca</strong></p></div></a>
</div>
<p class="note-acompte">Acompte de 50 % à la réservation · <a href="{{r}}politique-reservation/">Voir la politique de réservation</a></p>
</div><img class="deco" src="{{r}}images/fleurs.png" alt="" style="right:calc(50% - 364 * var(--u));bottom:calc(10 * var(--u));width:calc(100 * var(--u))"></section>'''
    page('tarifs/index.html', 'Tarifs · Norélias Photography', "Forfaits Essentiel 200 $, Signature 300 $, Complète 450 $, mini-séance de Noël 95 $ et forfait personnalisé. Campbellton, N.-B.", contenu, 'Tarifs')

# ---------------- Réservation ----------------
def reservation():
    opts = ''.join(f'<option data-code="{c}">{n}</option>' for n, c, _, _ in SEANCES if c != 'noel')
    heures = ''.join(f'<option>{h} h</option>' for h in range(9, 18))
    contenu = f'''<section class="page"><div class="interieur">
{titre_page("Réservation d'une séance", 'Choisissez votre type de séance, la date et réservez<br>en quelques clics.')}
<ol class="etapes"><li class="active"><span class="num">1</span>Prestation</li><li><span class="num">2</span>Date</li><li><span class="num">3</span>Informations</li><li><span class="num">4</span>Paiement</li><li><span class="num">5</span>Confirmation</li></ol>
<div class="zone-resa">
<form id="form-reservation" class="form-resa" novalidate>
  <fieldset class="etape" style="border:0;padding:0;margin:0"><legend class="sr-only">Étape 1 : prestation et date</legend>
    <div class="champ"><label class="sr-only" for="seance">Prestation</label><select id="seance" name="seance" required><option value="">Choisir une prestation</option>{opts}<option data-code="noel">Mini-séance de Noël</option></select></div>
    <div class="champ"><label class="sr-only" for="date">Date souhaitée</label><input type="text" id="date" name="date" placeholder="Date souhaitée" onfocus="this.type='date'" required></div>
    <div class="champ"><label class="sr-only" for="heure">Heure souhaitée</label><select id="heure" name="heure" required><option value="">Heure souhaitée</option>{heures}</select></div>
    <div class="navigation-etapes"><button type="button" class="bouton" data-suivant>Suivant {FLECHE}</button></div>
  </fieldset>
  <fieldset class="etape" hidden style="border:0;padding:0;margin:0"><legend class="sr-only">Étape 2 : informations</legend>
    <div class="deux-col">
      <div class="champ"><label class="sr-only" for="nom">Nom complet</label><input id="nom" name="nom" placeholder="Nom complet *" autocomplete="name" required></div>
      <div class="champ"><label class="sr-only" for="courriel">Courriel</label><input type="email" id="courriel" name="courriel" placeholder="Courriel *" autocomplete="email" required></div>
      <div class="champ"><label class="sr-only" for="telephone">Téléphone</label><input type="tel" id="telephone" name="telephone" placeholder="Téléphone *" autocomplete="tel" required></div>
      <div class="champ"><label class="sr-only" for="personnes">Nombre de personnes</label><input type="number" id="personnes" name="personnes" min="1" max="15" placeholder="Nombre de personnes" required></div>
    </div>
    <div class="champ"><label class="sr-only" for="message">Message</label><textarea id="message" name="message" rows="3" placeholder="Message (facultatif)"></textarea></div>
    <label class="case"><input type="checkbox" name="portfolio"> <span>J'accepte que certaines photos de ma séance soient utilisées dans le portfolio de Norélias (facultatif).</span></label>
    <div class="navigation-etapes"><button type="button" class="bouton contour" data-precedent>←</button><button type="button" class="bouton" data-suivant>Suivant {FLECHE}</button></div>
  </fieldset>
  <fieldset class="etape" hidden style="border:0;padding:0;margin:0"><legend class="sr-only">Étape 3 : paiement</legend>
    <div class="champ"><label class="sr-only" for="forfait">Forfait</label><select id="forfait" name="forfait" required><option value="">Choisir un forfait</option><option value="essentiel">Essentiel · 200 $</option><option value="signature">Signature · 300 $</option><option value="complete">Complète · 450 $</option><option value="noel">Mini-séance de Noël · 95 $</option><option value="personnalise">Forfait personnalisé · sur demande</option></select></div>
    <div class="recap"><dl><dt>Séance</dt><dd id="recap-seance"></dd><dt>Date</dt><dd id="recap-date"></dd></dl><dl class="recap-prix" hidden><dt>Forfait</dt><dd id="recap-forfait"></dd><dt>Total</dt><dd id="recap-total"></dd><dt>À payer maintenant</dt><dd id="recap-acompte"></dd></dl></div>
    <div id="zone-paiement" hidden>
      <div class="choix-paiement">
        <label><input type="radio" name="mode" value="carte" required> <span><strong>Carte de crédit</strong><br><span class="aide">Paiement sécurisé par Stripe</span></span></label>
        <label><input type="radio" name="mode" value="interac"> <span><strong>Virement Interac</strong><br><span class="aide">À envoyer dans les 48 h</span></span></label>
      </div>
      <div id="boite-interac" class="boite-interac" hidden>Envoyez <strong class="montant-interac"></strong> par virement Interac à <strong class="courriel-interac">bonjour@norelias.ca</strong> dans les 48 h, avec votre nom en message.</div>
    </div>
    <p id="note-perso" class="aide" hidden>Pour un forfait personnalisé, je vous réponds avec une proposition et le montant de l'acompte.</p>
    <div class="navigation-etapes"><button type="button" class="bouton contour" data-precedent>←</button><button type="submit" class="bouton" id="bouton-confirmer">Confirmer et payer</button></div>
  </fieldset>
  <div class="etape centre" hidden>
    <span class="script" style="font-size:calc(40 * var(--u));color:var(--rose-fonce)">Merci !</span>
    <h2 style="font-size:calc(26 * var(--u))">Votre demande est envoyée</h2>
    <div id="fin-carte" hidden><p>Vous allez être redirigée vers la page de paiement sécurisée Stripe. Si rien ne se passe, <a id="lien-stripe" href="#">cliquez ici pour payer</a>.</p></div>
    <div id="fin-interac" hidden><p>Pour confirmer votre date, envoyez <strong class="montant-interac"></strong> par virement Interac à <strong class="courriel-interac">bonjour@norelias.ca</strong> dans les 48 h.</p></div>
    <div id="fin-perso" hidden><p>Je vous réponds très bientôt avec une proposition personnalisée.</p></div>
    <p>Une question ? 506-787-0667</p>
  </div>
  <div class="bas-resa">
    <p class="note-resa">Acompte de 50 % à la réservation · Solde payable le jour de la séance</p>
    <label class="case"><input type="checkbox" name="politiques"> <span>J'accepte la <a href="{{r}}politique-reservation/" target="_blank">politique de réservation</a> et la <a href="{{r}}confidentialite/" target="_blank">politique de confidentialité</a></span></label>
  </div>
</form>
<img class="vase" src="{{r}}images/vase.png" alt="">
</div>
</div></section>'''
    page('reservation/index.html', 'Réserver une séance · Norélias Photography', "Réservez votre séance photo en ligne : grossesse, nouveau-né, famille, mini-séance de Noël. Acompte par carte ou virement Interac.", contenu, '')

# ---------------- Contact ----------------
def contact():
    contenu = f'''<section class="page"><div class="interieur">
{titre_page('Contact', 'Une question, un projet ou une envie de réserver une séance ?<br>Écrivez-moi, je réponds toujours avec plaisir !')}
<div class="contact">
  <div>
    <ul class="coordonnees">
      <li><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="5" width="18" height="14" rx="2"/><path d="M3 7l9 6 9-6"/></svg><a href="mailto:bonjour@norelias.ca" style="color:inherit;text-decoration:none">bonjour@norelias.ca</a></li>
      <li><svg viewBox="0 0 24 24" fill="currentColor"><path d="M6.6 10.8a15.1 15.1 0 0 0 6.6 6.6l2.2-2.2a1 1 0 0 1 1-.25 11.4 11.4 0 0 0 3.6.57 1 1 0 0 1 1 1V20a1 1 0 0 1-1 1A17 17 0 0 1 3 4a1 1 0 0 1 1-1h3.5a1 1 0 0 1 1 1c0 1.25.2 2.45.57 3.57a1 1 0 0 1-.25 1z"/></svg><a href="tel:+15067870667" style="color:inherit;text-decoration:none">506-787-0667</a></li>
      <li><svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2a7 7 0 0 0-7 7c0 5.3 7 13 7 13s7-7.7 7-13a7 7 0 0 0-7-7zm0 9.5A2.5 2.5 0 1 1 12 6.5a2.5 2.5 0 0 1 0 5z"/></svg><span>Campbellton, NB<br>et les environs</span></li>
      <li><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></svg><span>Tous les jours<br>9 h à 18 h</span></li>
    </ul>
    {reseaux()}
  </div>
  <div>
    <form id="form-contact">
      <div class="champ"><label class="sr-only" for="c-nom">Nom complet</label><input id="c-nom" name="nom" placeholder="Nom complet *" autocomplete="name" required></div>
      <div class="champ"><label class="sr-only" for="c-courriel">Courriel</label><input id="c-courriel" type="email" name="courriel" placeholder="Courriel *" autocomplete="email" required></div>
      <div class="champ"><label class="sr-only" for="c-sujet">Sujet</label><input id="c-sujet" name="sujet" placeholder="Sujet"></div>
      <div class="champ"><label class="sr-only" for="c-message">Message</label><textarea id="c-message" name="message" rows="5" placeholder="Message *" required></textarea></div>
      <button class="bouton" type="submit" style="width:100%">Envoyer mon message {FLECHE}</button>
    </form>
    <div id="contact-merci" class="merci"><span class="script" style="font-size:2.6rem;color:var(--rose-fonce)">Merci !</span><p>Votre message est bien parti. Je vous réponds très bientôt.</p></div>
  </div>
</div>
</div></section>'''
    page('contact/index.html', 'Contact · Norélias Photography', "Joindre Sabrina, Norélias Photography : 506-787-0667, bonjour@norelias.ca. Campbellton, N.-B., tous les jours de 9 h à 18 h.", contenu, 'Contact')

# ---------------- Politiques ----------------
def politique():
    blocs = [
        ('Réservation et paiement', ["Un acompte de 50 % réserve votre date. Il est non remboursable et déduit du total.", "Le solde est payable le jour de la séance (carte, Interac ou comptant).", "Mini-séances de Noël : paiement complet à la réservation."]),
        ('Report et annulation', ["Un report gratuit si vous m'avisez au moins 48 h à l'avance. L'acompte est transféré.", "En cas d'annulation, l'acompte n'est pas remboursé.", "Moins de 48 h ou absence : l'acompte est perdu. Un nouvel acompte est requis pour une autre date."]),
        ('Bébé, santé et météo', ["Nouveau-né : date provisoire selon votre date prévue, ajustée à la naissance. Avisez-moi dans les 48 h.", "Bébé ou un proche malade : report sans frais. La santé passe avant tout.", "Météo défavorable pour une séance extérieure : report sans frais."]),
        ('Livraison et droits', ["Vos photos en fichiers numériques haute résolution, à télécharger dans une galerie privée, dans les 3 semaines.", "Seules les photos retouchées sont livrées, dans le style de mon portfolio.", "Je garde les droits d'auteur. Vos photos vont dans mon portfolio seulement avec votre accord."]),
    ]
    html = ''.join('<div class="bloc-politique"><h3>' + t + '</h3><ul>' + ''.join(f'<li>{x}</li>' for x in items) + '</ul></div>' for t, items in blocs)
    contenu = f'''<section class="page"><div class="interieur">
{titre_page("Politique de réservation et d'annulation", 'Pour que chaque séance se passe en toute sérénité, pour vous comme pour moi.')}
<div class="grille-politique">{html}</div>
<p class="note-acompte" style="color:#7a4a40;font-size:1rem">Annulation de ma part (urgence, maladie) : report en priorité ou remboursement complet, à votre choix.</p>
</div></section>'''
    page('politique-reservation/index.html', "Politique de réservation et d'annulation · Norélias Photography", "Acompte de 50 %, report, annulation, santé de bébé, livraison des photos numériques : la politique de réservation de Norélias Photography.", contenu, '')

def confidentialite():
    contenu = f'''<section class="page"><div class="interieur texte-legal">
{titre_page('Politique de confidentialité')}
<p>Norélias Photography (Sabrina, Campbellton, N.-B.) respecte votre vie privée et celle de vos enfants. Cette page explique quels renseignements je recueille et comment je les utilise.</p>
<h2>Renseignements recueillis</h2><p>Quand vous réservez ou m'écrivez : votre nom, votre courriel, votre téléphone, la date souhaitée et les détails de votre séance.</p>
<h2>Utilisation</h2><p>Ces renseignements servent uniquement à organiser votre séance, à vous joindre et à vous livrer vos photos. Je ne les vends pas et je ne les partage pas, sauf avec les services nécessaires au site (voir plus bas).</p>
<h2>Paiements</h2><p>Les paiements par carte passent par Stripe, une plateforme sécurisée. Je n'ai jamais accès à votre numéro de carte. Les virements Interac passent par votre institution financière.</p>
<h2>Vos photos</h2><p>Vos photos sont livrées dans une galerie privée. Elles sont publiées dans mon portfolio ou sur mes réseaux sociaux seulement avec votre accord, que vous pouvez retirer en tout temps.</p>
<h2>Services utilisés</h2><p>Le site est hébergé par GitHub Pages et utilise Google Fonts. Les demandes peuvent être transmises par un service d'envoi de courriels (EmailJS). Ces services peuvent traiter des données techniques comme votre adresse IP.</p>
<h2>Vos droits</h2><p>Vous pouvez demander à consulter, corriger ou supprimer vos renseignements : <a href="mailto:bonjour@norelias.ca">bonjour@norelias.ca</a> ou 506-787-0667.</p>
<p class="aide">Dernière mise à jour : novembre 2026</p>
</div></section>'''
    page('confidentialite/index.html', 'Politique de confidentialité · Norélias Photography', "Comment Norélias Photography protège vos renseignements personnels et les photos de votre famille.", contenu, '')

def page404():
    contenu = f'''<section class="page centre"><div class="interieur"><span class="script" style="font-size:3.4rem;color:var(--rose-fonce)">Oups !</span><h1>Cette page n'existe pas</h1><p style="margin-top:24px"><a class="bouton" href="/">Retour à l'accueil {FLECHE}</a></p></div></section>'''
    page('404.html', 'Page introuvable · Norélias Photography', 'Page introuvable', contenu, '')

def css():
    src = open(os.path.join(RACINE, 'style.src.css'), encoding='utf-8').read()
    import random
    random.seed(11)
    x, y, pts = 0, 13.0, []
    while x < 1200:
        pts.append(f'L{x} {y:.1f}'); x += random.randint(4, 14); y += random.uniform(-3.5, 3.5); y = max(4, min(20, y * 0.9 + 1.3))
    chemin = 'M0 13 ' + ' '.join(pts[1:]) + ' L1200 12 L1200 26 L0 26 Z'
    svg = f"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 1200 26' preserveAspectRatio='none'><path d='{chemin}'/></svg>"
    src = src.replace(':root{', ':root{\n  --masque:url("data:image/svg+xml,' + svg.replace('<', '%3C').replace('>', '%3E') + '");', 1)
    out = re.sub(r'(?<![\w.#-])(-?\d+(?:\.\d+)?)u\b', lambda m: f'calc({m.group(1)} * var(--u))', src)
    open(os.path.join(RACINE, 'style.css'), 'w', encoding='utf-8').write('/* Fichier généré à partir de style.src.css — ne pas modifier directement */\n' + out)

if __name__ == '__main__':
    css()
    accueil(); prestations(); portfolio(); apropos(); tarifs(); reservation(); contact(); politique(); confidentialite(); page404()
    urls = ['', 'a-propos/', 'prestations/', 'portfolio/', 'tarifs/', 'reservation/', 'contact/', 'politique-reservation/', 'confidentialite/']
    with open(os.path.join(RACINE, 'sitemap.xml'), 'w') as f:
        f.write('<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n' + ''.join(f'  <url><loc>{SITE}/{u}</loc></url>\n' for u in urls) + '</urlset>\n')
    print('Pages générées.')
