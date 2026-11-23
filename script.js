/* Norélias Photography — comportements du site */
(function () {
  'use strict';
  var C = window.NORELIAS || {};

  /* ---------- Menu mobile ---------- */
  var burger = document.querySelector('.burger');
  var nav = document.querySelector('.nav');
  if (burger && nav) {
    burger.addEventListener('click', function () {
      var ouvert = nav.classList.toggle('ouvert');
      burger.setAttribute('aria-expanded', ouvert ? 'true' : 'false');
    });
  }

  /* ---------- Sous-menu Prestations (s'ouvre avec la flèche) ---------- */
  document.querySelectorAll('.ouvrir-sous').forEach(function (b) {
    b.addEventListener('click', function () {
      var li = b.parentNode, ouvert = li.classList.toggle('ouvert');
      b.setAttribute('aria-expanded', ouvert ? 'true' : 'false');
    });
  });

  /* ---------- Année du pied de page ---------- */
  document.querySelectorAll('.annee').forEach(function (el) { el.textContent = new Date().getFullYear(); });

  /* ---------- Filtres du portfolio ---------- */
  var filtres = document.querySelectorAll('.filtre');
  filtres.forEach(function (btn) {
    btn.addEventListener('click', function () {
      var cat = btn.dataset.cat;
      filtres.forEach(function (b) { b.setAttribute('aria-pressed', b === btn ? 'true' : 'false'); });
      document.querySelectorAll('.galerie figure').forEach(function (fig) {
        fig.hidden = !(cat === 'tous' || fig.dataset.cat === cat);
      });
    });
  });

  /* ---------- Portfolio : photo en grand, glisser ou flèches ---------- */
  var figs = Array.prototype.slice.call(document.querySelectorAll('.galerie figure'));
  if (figs.length) {
    var vis = document.createElement('div');
    vis.className = 'visionneuse'; vis.hidden = true;
    vis.setAttribute('role', 'dialog'); vis.setAttribute('aria-modal', 'true'); vis.setAttribute('aria-label', 'Photo en grand');
    vis.innerHTML = '<button type="button" class="v-fermer" aria-label="Fermer">×</button>' +
      '<button type="button" class="v-prec" aria-label="Photo précédente">‹</button>' +
      '<img alt=""><p class="v-legende"></p>' +
      '<button type="button" class="v-suiv" aria-label="Photo suivante">›</button>';
    document.body.appendChild(vis);
    var vImg = vis.querySelector('img'), vLeg = vis.querySelector('.v-legende'), liste = [], pos = 0, retour = null;
    function montrer(i) {
      pos = (i + liste.length) % liste.length;
      var im = liste[pos].querySelector('img');
      vImg.src = im.currentSrc || im.src; vImg.alt = im.alt;
      vLeg.textContent = im.alt + '  ·  ' + (pos + 1) + ' / ' + liste.length;
    }
    function ouvrir(fig) {
      liste = figs.filter(function (f) { return !f.hidden; });
      retour = document.activeElement;
      vis.hidden = false; document.body.style.overflow = 'hidden';
      montrer(liste.indexOf(fig));
      vis.querySelector('.v-fermer').focus();
    }
    function fermer() { vis.hidden = true; document.body.style.overflow = ''; if (retour) retour.focus(); }
    figs.forEach(function (fig) {
      fig.tabIndex = 0; fig.setAttribute('role', 'button');
      fig.setAttribute('aria-label', 'Agrandir : ' + (fig.querySelector('img') || {}).alt);
      fig.addEventListener('click', function () { ouvrir(fig); });
      fig.addEventListener('keydown', function (e) { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); ouvrir(fig); } });
    });
    vis.querySelector('.v-prec').addEventListener('click', function (e) { e.stopPropagation(); montrer(pos - 1); });
    vis.querySelector('.v-suiv').addEventListener('click', function (e) { e.stopPropagation(); montrer(pos + 1); });
    vis.querySelector('.v-fermer').addEventListener('click', fermer);
    vis.addEventListener('click', function (e) { if (e.target === vis) fermer(); });
    document.addEventListener('keydown', function (e) {
      if (vis.hidden) return;
      if (e.key === 'Escape') fermer();
      else if (e.key === 'ArrowLeft') montrer(pos - 1);
      else if (e.key === 'ArrowRight') montrer(pos + 1);
    });
    var x0 = null, y0 = null;
    vis.addEventListener('touchstart', function (e) { x0 = e.touches[0].clientX; y0 = e.touches[0].clientY; }, { passive: true });
    vis.addEventListener('touchend', function (e) {
      if (x0 === null) return;
      var dx = e.changedTouches[0].clientX - x0, dy = e.changedTouches[0].clientY - y0;
      if (Math.abs(dx) > 40 && Math.abs(dx) > Math.abs(dy)) montrer(pos + (dx < 0 ? 1 : -1));
      x0 = null;
    });
  }

  /* ---------- Envoi de courriel (EmailJS ou application courriel) ---------- */
  function emailjsPret(modele) {
    return C.emailjs && C.emailjs.service && C.emailjs.clePublique && modele && typeof window.emailjs !== 'undefined';
  }
  function envoyer(modele, donnees, sujet, corps) {
    if (emailjsPret(modele)) {
      return window.emailjs.send(C.emailjs.service, modele, donnees, C.emailjs.clePublique);
    }
    window.location.href = 'mailto:' + C.courriel + '?subject=' + encodeURIComponent(sujet) + '&body=' + encodeURIComponent(corps);
    return Promise.resolve();
  }

  /* ---------- Formulaire de contact ---------- */
  var formContact = document.getElementById('form-contact');
  if (formContact) {
    formContact.addEventListener('submit', function (e) {
      e.preventDefault();
      var d = {
        nom: formContact.elements.nom.value.trim(),
        courriel: formContact.elements.courriel.value.trim(),
        sujet: formContact.elements.sujet.value.trim() || 'Message du site Norélias',
        message: formContact.elements.message.value.trim()
      };
      var corps = d.message + '\n\n— ' + d.nom + ' (' + d.courriel + ')';
      var bouton = formContact.querySelector('button[type="submit"]');
      bouton.disabled = true;
      envoyer(C.emailjs && C.emailjs.modeleContact, d, d.sujet, corps).then(function () {
        formContact.style.display = 'none';
        document.getElementById('contact-merci').style.display = 'block';
      }).catch(function () {
        alert("Oups, le message n'a pas pu partir. Écrivez-moi à " + C.courriel + ' ou appelez-moi au ' + C.telephone + '.');
        bouton.disabled = false;
      });
    });
  }

  /* ---------- Réservation en 5 étapes ---------- */
  var form = document.getElementById('form-reservation');
  if (!form) return;

  var FORFAITS = {
    essentiel: { nom: 'Essentiel', prix: 200, acompte: 100 },
    signature: { nom: 'Signature', prix: 300, acompte: 150 },
    complete: { nom: 'Complète', prix: 450, acompte: 225 },
    noel: { nom: 'Mini-séance de Noël', prix: 95, acompte: 95 },
    ecole: { nom: 'Garderie / École', prix: 35, parEnfant: true, minimum: 10 },
    finissants: { nom: 'Finissants', prix: 175, acompte: 87.5 },
    bal: { nom: 'Bal de promos', prix: 150, parPersonneEnPlus: 25, inclus: 2 },
    personnalise: { nom: 'Forfait personnalisé', prix: null, acompte: 100 }
  };
  var etapes = form.querySelectorAll('.etape');
  var puces = document.querySelectorAll('.etapes li');
  var courante = 0;

  // Préremplir depuis l'adresse (ex. reservation/?seance=noel)
  var params = new URLSearchParams(window.location.search);
  if (params.get('seance')) {
    var s = params.get('seance');
    if (s === 'noel') { form.elements.seance.value = 'Mini-séance de Noël'; form.elements.forfait.value = 'noel'; }
    else {
      Array.prototype.forEach.call(form.elements.seance.options, function (o) { if (o.dataset.code === s) form.elements.seance.value = o.value; });
    }
  }
  if (params.get('forfait') && FORFAITS[params.get('forfait')]) form.elements.forfait.value = params.get('forfait');

  // Séances qui ont leur propre prix : le forfait est choisi automatiquement
  var DEDIE = { 'noel': 'noel', 'garderie-ecole': 'ecole', 'finissants': 'finissants', 'bal': 'bal' };
  var DEDIES = ['noel', 'ecole', 'finissants', 'bal'];
  function codeSeance() {
    var o = form.elements.seance.options[form.elements.seance.selectedIndex];
    return o ? o.dataset.code : '';
  }
  function synchroniserNoel() {
    var dedie = DEDIE[codeSeance()];
    if (dedie) form.elements.forfait.value = dedie;
    Array.prototype.forEach.call(form.elements.forfait.options, function (o) {
      if (!o.value) return;
      o.disabled = dedie ? o.value !== dedie : DEDIES.indexOf(o.value) > -1;
    });
    if (!dedie && DEDIES.indexOf(form.elements.forfait.value) > -1) form.elements.forfait.value = '';
    var pers = form.elements.personnes, ecole = form.elements.ecole, groupes = form.elements.groupes;
    var avecEcole = dedie === 'ecole' || dedie === 'bal';
    document.getElementById('champ-ecole').hidden = !avecEcole; ecole.required = avecEcole;
    document.getElementById('champ-groupes').hidden = dedie !== 'ecole'; groupes.required = dedie === 'ecole';
    ecole.placeholder = dedie === 'bal' ? "Nom de l'école *" : "Nom de l'école ou de la garderie *";
    if (dedie === 'ecole') { pers.placeholder = "Nombre total d'élèves ou d'enfants (minimum 10) *"; pers.min = 10; pers.max = 2000; }
    else if (dedie === 'bal') { pers.placeholder = 'Nombre de personnes sur les photos *'; pers.min = 1; pers.max = 40; }
    else { pers.placeholder = 'Nombre de personnes *'; pers.min = 1; pers.max = 15; }
    majCalcul();
    if (window.majCalendrier) window.majCalendrier();
  }
  form.elements.seance.addEventListener('change', synchroniserNoel);


  /* ---------- Mini calendrier des disponibilités ---------- */
  var D = C.disponibilites || {};
  var cal = document.getElementById('calendrier');
  var MOIS = ['janvier','février','mars','avril','mai','juin','juillet','août','septembre','octobre','novembre','décembre'];
  var JOURS = ['dimanche','lundi','mardi','mercredi','jeudi','vendredi','samedi'];
  function iso(d) { return d.getFullYear() + '-' + ('0' + (d.getMonth() + 1)).slice(-2) + '-' + ('0' + d.getDate()).slice(-2); }
  function estNoel() { return form.elements.seance.value === 'Mini-séance de Noël'; }
  function heuresDu(d) {
    var cle = iso(d);
    if ((D.joursFermes || []).indexOf(cle) > -1) return [];
    if (estNoel() && d.getMonth() !== 11) return [];
    var prises = (D.dejaReserve || {})[cle] || [];
    if (prises.indexOf('tout') > -1) return [];
    var liste = (estNoel() ? D.heuresNoel : D.heures) || ['9 h', '10 h 30', '13 h', '14 h 30', '16 h'];
    return liste.filter(function (h) { return prises.indexOf(h) === -1; });
  }
  var debut = new Date(); debut.setHours(0, 0, 0, 0); debut.setDate(debut.getDate() + (D.delaiMinimum == null ? 2 : D.delaiMinimum));
  var fin = new Date(debut); fin.setDate(fin.getDate() + (D.joursAffiches || 120));
  var vue = new Date(debut.getFullYear(), debut.getMonth(), 1);
  var choisie = null;

  function premiereDispo() {
    for (var d = new Date(debut); d <= fin; d.setDate(d.getDate() + 1)) if (heuresDu(d).length) return new Date(d);
    return null;
  }
  function dessiner() {
    if (!cal) return;
    cal.querySelector('.cal-mois').textContent = MOIS[vue.getMonth()] + ' ' + vue.getFullYear();
    var grille = cal.querySelector('.cal-grille'); grille.innerHTML = '';
    var decal = new Date(vue.getFullYear(), vue.getMonth(), 1).getDay();
    for (var k = 0; k < decal; k++) grille.appendChild(document.createElement('span'));
    var nb = new Date(vue.getFullYear(), vue.getMonth() + 1, 0).getDate();
    for (var j = 1; j <= nb; j++) {
      var d = new Date(vue.getFullYear(), vue.getMonth(), j);
      var dispo = d >= debut && d <= fin && heuresDu(d).length > 0;
      var el = document.createElement(dispo ? 'button' : 'span');
      el.textContent = j;
      if (dispo) {
        el.type = 'button';
        el.dataset.iso = iso(d);
        el.setAttribute('aria-label', JOURS[d.getDay()] + ' ' + j + ' ' + MOIS[d.getMonth()] + ', disponible');
        el.setAttribute('aria-pressed', choisie && iso(choisie) === iso(d) ? 'true' : 'false');
        el.addEventListener('click', (function (dd) { return function () { choisirDate(dd); }; })(d));
      }
      grille.appendChild(el);
    }
    cal.querySelector('[data-mois="-1"]').disabled = vue <= new Date(debut.getFullYear(), debut.getMonth(), 1);
    cal.querySelector('[data-mois="1"]').disabled = new Date(vue.getFullYear(), vue.getMonth() + 1, 1) > fin;
  }
  var champHeure = form.elements.heure, boutonCal = document.getElementById('ouvrir-cal');
  function ouvrirCal(ouvert) {
    if (!cal) return;
    cal.hidden = !ouvert; boutonCal.setAttribute('aria-expanded', ouvert ? 'true' : 'false');
  }
  function remplirHeures(d) {
    champHeure.innerHTML = '<option value="">Heure souhaitée</option>';
    if (!d) { champHeure.disabled = true; return; }
    heuresDu(d).forEach(function (h) { var o = document.createElement('option'); o.textContent = h; champHeure.appendChild(o); });
    champHeure.disabled = false;
  }
  function choisirDate(d, garderOuvert) {
    choisie = d;
    form.elements.date.value = JOURS[d.getDay()] + ' ' + d.getDate() + ' ' + MOIS[d.getMonth()] + ' ' + d.getFullYear();
    form.elements.dateIso.value = iso(d);
    boutonCal.querySelector('.valeur-date').textContent = form.elements.date.value;
    boutonCal.classList.add('rempli');
    document.querySelector('.erreur-cal').hidden = true;
    remplirHeures(d);
    dessiner();
    if (!garderOuvert) { ouvrirCal(false); champHeure.focus(); }
  }
  if (boutonCal) {
    boutonCal.addEventListener('click', function () { ouvrirCal(cal.hidden); });
    document.addEventListener('click', function (e) { if (!e.target.closest('.champ-date')) ouvrirCal(false); });
    document.addEventListener('keydown', function (e) { if (e.key === 'Escape') ouvrirCal(false); });
  }
  window.majCalendrier = function () {
    if (!cal) return;
    if (choisie && !heuresDu(choisie).length) {
      choisie = null; form.elements.date.value = ''; form.elements.dateIso.value = ''; remplirHeures(null);
      boutonCal.querySelector('.valeur-date').textContent = 'Date souhaitée'; boutonCal.classList.remove('rempli');
    }
    var p = premiereDispo();
    if (!choisie && p) vue = new Date(p.getFullYear(), p.getMonth(), 1);
    if (choisie) { var h = champHeure.value; choisirDate(choisie, true); if (heuresDu(choisie).indexOf(h) > -1) champHeure.value = h; } else dessiner();
  };
  if (cal) cal.querySelectorAll('.cal-nav').forEach(function (b) {
    b.addEventListener('click', function () { vue = new Date(vue.getFullYear(), vue.getMonth() + Number(b.dataset.mois), 1); dessiner(); });
  });
  synchroniserNoel();

  /* ---------- Serveur de réservations ---------- */
  var idBrouillon = null;
  try { idBrouillon = sessionStorage.getItem('norelias-brouillon'); } catch (e) {}
  function codeSeanceChoisie() { var o = form.elements.seance.options[form.elements.seance.selectedIndex]; return o ? o.dataset.code || '' : ''; }
  function donneesServeur() {
    var portfolio = form.querySelector('input[name="portfolio"]:checked');
    return {
      id: idBrouillon, seance: form.elements.seance.value, seance_code: codeSeanceChoisie(), forfait: form.elements.forfait.value,
      date_iso: form.elements.dateIso.value, date_txt: form.elements.date.value, heure: form.elements.heure.value,
      nom: form.elements.nom.value.trim(), courriel: form.elements.courriel.value.trim(), telephone: form.elements.telephone.value.trim(),
      personnes: form.elements.personnes.value, message: form.elements.message.value.trim(), portfolio: portfolio ? portfolio.value : '',
      ecole: form.elements.ecole.value.trim(), groupes: form.elements.groupes.value
    };
  }
  function appelServeur(corps) {
    return fetch(C.serveur, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(corps) })
      .then(function (r) { return r.json().then(function (j) { if (!r.ok || j.ok === false) { var e = new Error(j.erreur || 'erreur'); e.message = j.erreur || ''; e.statut = r.status; throw e; } return j; }); });
  }
  function sauverBrouillon() {
    if (!C.serveur || !window.fetch) return;
    var d = donneesServeur(); d.action = 'brouillon';
    appelServeur(d).then(function (j) { if (j.id) { idBrouillon = j.id; try { sessionStorage.setItem('norelias-brouillon', j.id); } catch (e) {} } }).catch(function () {});
  }
  // Plages déjà réservées par d'autres clientes : on les retire du calendrier
  if (C.serveur && window.fetch) {
    fetch(C.serveur).then(function (r) { return r.json(); }).then(function (j) {
      var p = j.prises || {}; D.dejaReserve = D.dejaReserve || {};
      Object.keys(p).forEach(function (k) { D.dejaReserve[k] = (D.dejaReserve[k] || []).concat(p[k]); });
      if (window.majCalendrier) window.majCalendrier();
    }).catch(function () {});
  }

  function majCalcul() {
    var zone = document.getElementById('calcul-prix'); if (!zone) return;
    var cle = form.elements.forfait.value;
    if ((cle !== 'ecole' && cle !== 'bal') || !form.elements.personnes.value) { zone.hidden = true; return; }
    var m = montants(cle);
    zone.hidden = false;
    zone.textContent = 'Total : ' + argent(m.total) + (m.detail ? ' (' + m.detail + ')' : '') + ' · Acompte : ' + argent(m.acompte);
  }
  form.elements.personnes.addEventListener('input', majCalcul);

  function argent(n) { return n.toLocaleString('fr-CA', { minimumFractionDigits: n % 1 ? 2 : 0, maximumFractionDigits: 2 }) + ' $'; }
  // Total et acompte réels (garderie / école : 35 $ par enfant, minimum 10)
  function montants(cle) {
    var f = FORFAITS[cle];
    if (!f) return null;
    if (f.parEnfant) {
      var n = Math.max(f.minimum, parseInt(form.elements.personnes.value, 10) || 0);
      var total = f.prix * n;
      return { total: total, acompte: Math.round(total * 50) / 100, detail: n + ' enfants × ' + argent(f.prix) };
    }
    if (f.parPersonneEnPlus) {
      var p = Math.max(1, parseInt(form.elements.personnes.value, 10) || 1), extra = Math.max(0, p - f.inclus);
      var t = f.prix + extra * f.parPersonneEnPlus;
      return { total: t, acompte: Math.round(t * 50) / 100, detail: extra ? argent(f.prix) + ' + ' + extra + ' × ' + argent(f.parPersonneEnPlus) : '' };
    }
    return { total: f.prix, acompte: f.acompte };
  }

  function afficher(i) {
    courante = i;
    etapes.forEach(function (el, k) { el.hidden = k !== i; });
    var PUCES = [[0, 1], [2], [3], [4]];
    var actives = PUCES[i], premiere = actives[0];
    puces.forEach(function (li, k) {
      var act = actives.indexOf(k) > -1;
      li.classList.toggle('active', act);
      li.classList.toggle('faite', k < premiere);
      if (act) li.setAttribute('aria-current', 'step'); else li.removeAttribute('aria-current');
    });
    document.querySelector('.bas-resa').hidden = i === 3;
    if (i === 2) preparerPaiement();
    form.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  function valider(i) {
    if (i === 0 && !form.elements.seance.checkValidity()) { form.elements.seance.reportValidity(); return false; }
    if (i === 0 && !form.elements.date.value) {
      document.querySelector('.erreur-cal').hidden = false; ouvrirCal(true); return false;
    }
    var champs = etapes[i].querySelectorAll('input,select,textarea');
    for (var k = 0; k < champs.length; k++) {
      if (!champs[k].checkValidity()) { champs[k].reportValidity(); return false; }
    }
    return true;
  }

  form.querySelectorAll('[data-suivant]').forEach(function (b) {
    b.addEventListener('click', function () {
      if (!valider(courante)) return;
      if (courante === 1) sauverBrouillon(); // réservation commencée : rappel si elle n'est pas terminée
      afficher(courante + 1);
    });
  });
  form.querySelectorAll('[data-precedent]').forEach(function (b) {
    b.addEventListener('click', function () { afficher(courante - 1); });
  });

  function preparerPaiement() {
    var f = FORFAITS[form.elements.forfait.value];
    document.getElementById('recap-seance').textContent = form.elements.seance.value;
    document.getElementById('recap-date').textContent = form.elements.date.value + ' · ' + form.elements.heure.value;
    document.querySelector('.recap-prix').hidden = !f;
    if (!f) { document.getElementById('zone-paiement').hidden = true; return; }
    var perso = false, estPerso = form.elements.forfait.value === 'personnalise';
    document.getElementById('recap-seance').textContent = form.elements.seance.value;
    document.getElementById('recap-forfait').textContent = f.nom;
    document.getElementById('recap-date').textContent = form.elements.date.value + ' · ' + form.elements.heure.value;
    var m = montants(form.elements.forfait.value);
    document.getElementById('recap-total').textContent = estPerso ? 'Sur demande' : argent(m.total) + (m.detail ? ' (' + m.detail + ')' : '');
    var acompte = document.getElementById('recap-acompte');
    acompte.textContent = estPerso ? argent(m.acompte) + ' (dépôt déduit du total)' : (form.elements.forfait.value === 'noel' ? argent(m.acompte) + ' (paiement complet)' : argent(m.acompte) + ' (50 %)');
    document.getElementById('zone-paiement').hidden = perso;
    document.getElementById('note-perso').hidden = !estPerso;
    form.querySelectorAll('input[name="mode"]').forEach(function (r) { r.required = !perso; });
    document.getElementById('bouton-confirmer').textContent = perso ? 'Envoyer ma demande' : 'Confirmer et payer';
    majInterac();
  }

  function majInterac() {
    var f = FORFAITS[form.elements.forfait.value];
    var boite = document.getElementById('boite-interac');
    var choix = form.querySelector('input[name="mode"]:checked');
    boite.hidden = !(choix && choix.value === 'interac');
    if (f) {
      var m = montants(form.elements.forfait.value);
      document.querySelectorAll('.montant-interac').forEach(function (el) { el.textContent = argent(m.acompte); });
    }
  }
  form.querySelectorAll('input[name="mode"]').forEach(function (r) { r.addEventListener('change', majInterac); });
  form.elements.forfait.addEventListener('change', function () { if (form.elements.forfait.value) preparerPaiement(); });

  form.addEventListener('submit', function (e) {
    e.preventDefault();
    if (!valider(2)) return;
    if (!form.elements.politiques.checked) { form.elements.politiques.reportValidity(); return; }
    var cle = form.elements.forfait.value;
    var f = FORFAITS[cle];
    var perso = false;
    var modeEl = form.querySelector('input[name="mode"]:checked');
    var mode = perso ? 'aucun' : modeEl.value;
    var lienStripe = (C.stripe && C.stripe[cle]) || '';
    if (mode === 'carte' && !C.paiementCarteActif && !lienStripe) mode = 'interac'; // paiement par carte pas encore activé

    var d = {
      seance: form.elements.seance.value,
      forfait: f.nom,
      date: form.elements.date.value,
      heure: form.elements.heure.value,
      nom: form.elements.nom.value.trim(),
      courriel: form.elements.courriel.value.trim(),
      telephone: form.elements.telephone.value.trim(),
      personnes: form.elements.personnes.value,
      message: form.elements.message.value.trim(),
      portfolio: (form.querySelector('input[name="portfolio"]:checked') || {}).value || 'Non',
      paiement: mode === 'carte' ? 'Carte (Stripe)' : (mode === 'interac' ? 'Virement Interac' : 'À discuter'),
      montant: argent(montants(cle).acompte) + (montants(cle).total ? ' (total ' + argent(montants(cle).total) + ')' : ' (dépôt, total sur demande)')
    };
    var corps = [
      'Nouvelle demande de réservation — Norélias Photography', '',
      'Séance : ' + d.seance, 'Forfait : ' + d.forfait, 'Date : ' + d.date + ' à ' + d.heure, '',
      'Nom : ' + d.nom, 'Courriel : ' + d.courriel, 'Téléphone : ' + d.telephone, 'Nombre de personnes : ' + d.personnes, '',
      'Paiement : ' + d.paiement + ' — ' + d.montant,
      'Photos dans le portfolio : ' + d.portfolio,
      'Politiques acceptées : Oui', '', 'Message : ' + (d.message || '—')
    ].join('\n');

    var bouton = document.getElementById('bouton-confirmer');
    bouton.disabled = true;
    var envoi;
    if (C.serveur && window.fetch) {
      var ds = donneesServeur(); ds.action = 'soumettre'; ds.mode = mode;
      envoi = appelServeur(ds).then(function (j) { try { sessionStorage.removeItem('norelias-brouillon'); } catch (e) {} if (j && j.paiement) lienStripe = j.paiement; });
    } else {
      envoi = Promise.reject(new Error('serveur indisponible'));
    }
    envoi.then(function () {
      document.getElementById('fin-carte').hidden = mode !== 'carte';
      document.getElementById('fin-interac').hidden = mode !== 'interac';
      document.getElementById('fin-perso').hidden = cle !== 'personnalise';
      afficher(3);
      if (mode === 'carte') {
        var url = lienStripe.indexOf('checkout.stripe.com') > -1 ? lienStripe : lienStripe + (lienStripe.indexOf('?') > -1 ? '&' : '?') + 'prefilled_email=' + encodeURIComponent(d.courriel);
        document.getElementById('lien-stripe').href = url;
        setTimeout(function () { window.location.href = url; }, 1500);
      }
    }).catch(function (e) {
      if (e && e.statut === 409) {
        alert(e.message);
        afficher(0); window.majCalendrier && window.majCalendrier();
      } else {
        alert("Oups, la demande n'a pas pu partir. Écrivez-moi à " + C.courriel + ' ou appelez-moi au ' + C.telephone + '.');
      }
      bouton.disabled = false;
    });
  });

  document.querySelectorAll('.courriel-interac').forEach(function (el) { el.textContent = C.interac; });

  // Retour de la page de paiement Stripe
  var qp = new URLSearchParams(location.search);
  if (qp.get('paye') === '1') {
    document.getElementById('fin-carte').hidden = false;
    document.getElementById('fin-carte').innerHTML = '<p><strong>Paiement reçu, merci !</strong> Vous recevrez un courriel de confirmation avec tous les détails.</p>';
    afficher(3);
  }
})();

  /* ---------- Carrousel des témoignages ---------- */
  (function () {
    var piste = document.querySelector('.grille-temoins');
    if (!piste) return;
    var prec = document.querySelector('.fl-t.prec'), suiv = document.querySelector('.fl-t.suiv');
    var pause = false;
    function pas() { var c = piste.querySelector('.temoin'); return c ? c.getBoundingClientRect().width + 14 : 300; }
    function aller(sens) {
      var fin = piste.scrollLeft + piste.clientWidth >= piste.scrollWidth - 4;
      if (sens > 0 && fin) { piste.scrollTo({ left: 0 }); return; }
      if (sens < 0 && piste.scrollLeft <= 2) { piste.scrollTo({ left: piste.scrollWidth }); return; }
      piste.scrollBy({ left: sens * pas() });
    }
    prec.addEventListener('click', function () { aller(-1); });
    suiv.addEventListener('click', function () { aller(1); });
    var zone = piste.parentNode;
    ['mouseenter', 'focusin', 'touchstart'].forEach(function (e) { zone.addEventListener(e, function () { pause = true; }, { passive: true }); });
    ['mouseleave', 'focusout'].forEach(function (e) { zone.addEventListener(e, function () { pause = false; }); });
    setInterval(function () { if (!pause && !document.hidden) aller(1); }, 6000);
  })();

/* ---------- Panier (stockage local) ---------- */
(function () {
  var CLE = 'norelias_panier';
  function lire() { try { var a = JSON.parse(localStorage.getItem(CLE) || '[]'); return Array.isArray(a) ? a : []; } catch (e) { return []; } }
  function ecrire(a) { try { localStorage.setItem(CLE, JSON.stringify(a)); } catch (e) { return false; } maj(); return true; }
  function n() { return lire().reduce(function (t, x) { return t + (Number(x.quantite) || 1); }, 0); }
  function maj() { Array.prototype.forEach.call(document.querySelectorAll('.panier-n'), function (e) { var k = n(); e.textContent = k; e.hidden = !k; }); }
  window.NBQ = { lire: lire, ecrire: ecrire, n: n, vider: function () { try { localStorage.removeItem(CLE); } catch (e) {} maj(); } };
  maj();
})();

/* ---------- Boutique ---------- */
(function () {
  var $ = function (id) { return document.getElementById(id); };
  // listing : filtres, favoris
  Array.prototype.forEach.call(document.querySelectorAll('[data-filtre]'), function (b) {
    b.addEventListener('click', function () {
      var v = b.getAttribute('data-filtre');
      Array.prototype.forEach.call(document.querySelectorAll('[data-filtre]'), function (x) { x.classList.toggle('actif', x === b); });
      Array.prototype.forEach.call(document.querySelectorAll('#grille-prod .prod'), function (a) { a.hidden = v !== 'tous' && a.getAttribute('data-cat') !== v; });
      var ap = document.querySelector('.apercu-vignette'); if (ap) ap.hidden = v !== 'tous';
    });
  });
  Array.prototype.forEach.call(document.querySelectorAll('.coeur-fav'), function (h) { h.addEventListener('click', function (e) { e.preventDefault(); e.stopPropagation(); h.classList.toggle('on'); }); });
  var q0 = location.search, m0 = $('msg-boutique');
  if (m0 && /merci=1/.test(q0)) { if (window.NBQ) window.NBQ.vider(); m0.textContent = 'Merci ! Votre paiement est reçu. Un courriel de confirmation vous attend.'; m0.hidden = false; }
  else if (m0 && /annule=1/.test(q0)) { m0.textContent = "Le paiement a été annulé. Votre commande n'a pas été complétée."; m0.hidden = false; }

  var f = $('form-produit');
  if (!f) return;
  var C = window.NORELIAS || {};
  var P = JSON.parse(f.getAttribute('data-produit'));
  var CODE = f.getAttribute('data-code'), LIV = Number(f.getAttribute('data-livraison'));
  var img = null, photoData = null, vue = 0, nomFichier = '', tailleFichier = '', idModif = null;
  var cad = { z: 1, x: 0, y: 0 };
  var txt = { t: '', police: 'Dancing Script', couleur: CODE === 'isotherme' ? '#5c2f2b' : '#ffffff', pos: 'devant' };
  var cvA = $('pp-apercu'), qte = $('pp-qte'), fi = $('pp-photo');
  function radios() { return f.querySelectorAll('input[name=variante]'); }
  function variante() { var r = f.querySelector('input[name=variante]:checked'); return r ? r.value : Object.keys(P.v)[0]; }
  function cover(ctx, im, x, y, w, h) {
    var r = Math.max(w / im.width, h / im.height) * cad.z, sw = w / r, sh = h / r;
    ctx.drawImage(im, (im.width - sw) / 2 * (1 + cad.x), (im.height - sh) / 2 * (1 + cad.y), sw, sh, x, y, w, h);
  }
  function ecrire(c, x, y, maxw) {
    if (!txt.t) return;
    c.save(); c.textAlign = 'center'; c.textBaseline = 'middle';
    var px = 36, pol = function () { c.font = px + 'px "' + txt.police + '", cursive'; }; pol();
    var w = c.measureText(txt.t).width; if (w > maxw) { px = Math.max(10, Math.floor(px * maxw / w)); pol(); }
    var h = txt.couleur.replace('#', ''), lum = (parseInt(h.substr(0, 2), 16) * .3 + parseInt(h.substr(2, 2), 16) * .59 + parseInt(h.substr(4, 2), 16) * .11);
    c.shadowColor = lum > 150 ? 'rgba(0,0,0,.5)' : 'rgba(255,255,255,.55)'; c.shadowBlur = 5; c.shadowOffsetY = 1;
    c.fillStyle = txt.couleur; c.fillText(txt.t, x, y); c.restore();
  }
  var racine = (document.querySelector('link[href*="style.css"]').getAttribute('href') || '').replace(/style\.css.*$/, '');

  /* ---- Aperçu réaliste : la photo de la cliente est posée sur une vraie photo du produit ---- */
  var BASES = {
    tasse: { cyl: 1, pret: true, W: 1000, H: 640, vues: [{ id: 'face' }, { id: 'dos' }, { id: 'table' }, { id: 'cadeau' }, { id: 'formats' }] },
    isotherme: { cyl: 1, pret: true, W: 1000, H: 880, vues: [{ id: 'face' }, { id: 'dos' }, { id: 'table' }, { id: 'cadeau' }, { id: 'formats' }] },
    rond: { src: 'images/pp-rond.jpg', rond: { x: 229, y: 359, r: 120 }, vues: [{}, { zoom: [229, 359, 1.9] }] },
    ornement: { src: 'images/pp-ornement.jpg', rond: { x: 194, y: 107, r: 63 }, vues: [{}, { zoom: [194, 107, 2] }] },
    proc: { vues: [{}, { zoom: [225, 335, 1.8] }], W: 681, H: 551, pret: true }
  };
  var vitesse = 0;
  Object.keys(BASES).forEach(function (k) { var b = BASES[k]; if (!b.src) return; b.img = new Image(); b.img.onload = function () { b.pret = true; b.W = b.img.width; b.H = b.img.height; if (window.__maj) window.__maj(); }; b.img.src = racine + b.src; });
  function cleBase(v) { if (CODE === 'portecles') return v === 'rond' ? 'rond' : 'proc'; return CODE; }

  function heart(c, cx, cy, s) {
    c.beginPath();
    for (var i = 0; i <= 80; i++) {
      var t = i / 80 * Math.PI * 2, x = Math.pow(Math.sin(t), 3), y = (13 * Math.cos(t) - 5 * Math.cos(2 * t) - 2 * Math.cos(3 * t) - Math.cos(4 * t)) / 16;
      if (i) c.lineTo(cx + x * s, cy - y * s + s * .1); else c.moveTo(cx + x * s, cy - y * s + s * .1);
    }
    c.closePath();
  }
  function rr(c, x, y, w, h, r) { c.beginPath(); c.moveTo(x + r, y); c.arcTo(x + w, y, x + w, y + h, r); c.arcTo(x + w, y + h, x, y + h, r); c.arcTo(x, y + h, x, y, r); c.arcTo(x, y, x + w, y, r); c.closePath(); }
  function argent(c, x0, y0, x1, y1) { var g = c.createLinearGradient(x0, y0, x1, y1); g.addColorStop(0, '#f7f7f7'); g.addColorStop(.3, '#b9bbbd'); g.addColorStop(.55, '#efeff0'); g.addColorStop(.8, '#8f9193'); g.addColorStop(1, '#d9dadb'); return g; }
  function anneau(c, x, y, r) {
    c.save(); c.lineWidth = 9; c.strokeStyle = argent(c, x - r, y - r, x + r, y + r); c.shadowColor = 'rgba(60,40,30,.3)'; c.shadowBlur = 8; c.shadowOffsetY = 4;
    c.beginPath(); c.arc(x, y, r, 0, 7); c.stroke(); c.restore();
  }
  // porte-clés dessinés (cœur, rectangle, simili cuir) : face avec la photo + dos
  function objet(c, v, im, cx, cy, face) {
    c.save(); c.shadowColor = 'rgba(70,45,35,.32)'; c.shadowBlur = 20; c.shadowOffsetX = 7; c.shadowOffsetY = 12;
    if (v === 'coeur') {
      var S = 120, hy = cy - S * .26;
      anneau(c, cx, hy - S * .64 - 22, 36);
      heart(c, cx, hy, S); c.fillStyle = argent(c, cx - S, hy - S, cx + S, hy + S); c.fill(); c.shadowColor = 'transparent';
      if (face) { c.save(); heart(c, cx, hy + 1, S * .9); c.clip(); if (im) cover(c, im, cx - S, hy - S * .66, S * 2, S * 1.84); else { c.fillStyle = '#d9c7bb'; c.fillRect(cx - S, hy - S, S * 2, S * 2.2); } if (im) ecrire(c, cx, hy + S * .42, S * 1.1); c.restore(); }
      else { heart(c, cx, hy + 1, S * .9); var g = c.createLinearGradient(cx - S, hy - S * .6, cx + S, hy + S); g.addColorStop(0, '#cfd0d2'); g.addColorStop(.5, '#e9eaeb'); g.addColorStop(1, '#b4b6b8'); c.fillStyle = g; c.fill(); }
    } else if (v === 'rect') {
      anneau(c, cx, cy - 188, 36); rr(c, cx - 22, cy - 160, 44, 36, 8); c.fillStyle = argent(c, cx - 22, cy - 160, cx + 22, cy - 124); c.fill();
      rr(c, cx - 100, cy - 128, 200, 262, 12); c.fillStyle = argent(c, cx - 100, cy - 128, cx + 100, cy + 134); c.fill(); c.shadowColor = 'transparent';
      if (face) { c.save(); rr(c, cx - 86, cy - 114, 172, 234, 6); c.clip(); if (im) cover(c, im, cx - 86, cy - 114, 172, 234); else { c.fillStyle = '#d9c7bb'; c.fillRect(cx - 90, cy - 118, 180, 242); } if (im) ecrire(c, cx, cy + 92, 150); c.restore(); }
      else { rr(c, cx - 86, cy - 114, 172, 234, 6); c.fillStyle = '#dcdddf'; c.fill(); }
    } else {
      anneau(c, cx, cy - 196, 36);
      rr(c, cx - 36, cy - 168, 72, 80, 14); c.fillStyle = '#1b191a'; c.fill();
      rr(c, cx - 104, cy - 112, 208, 262, 22); var g2 = c.createLinearGradient(cx - 104, cy - 112, cx + 104, cy + 150); g2.addColorStop(0, '#343032'); g2.addColorStop(.5, '#1b191a'); g2.addColorStop(1, '#2a2628'); c.fillStyle = g2; c.fill(); c.shadowColor = 'transparent';
      c.beginPath(); c.arc(cx, cy - 128, 12, 0, 7); c.fillStyle = argent(c, cx - 12, cy - 140, cx + 12, cy - 116); c.fill();
      c.setLineDash([5, 4]); c.lineWidth = 1.6; c.strokeStyle = '#8a8284'; rr(c, cx - 94, cy - 102, 188, 242, 16); c.stroke(); c.setLineDash([]);
      if (face) { c.save(); rr(c, cx - 70, cy - 66, 140, 180, 5); c.clip(); if (im) cover(c, im, cx - 70, cy - 66, 140, 180); else { c.fillStyle = '#d9c7bb'; c.fillRect(cx - 72, cy - 68, 144, 184); } if (im) ecrire(c, cx, cy + 96, 120); c.restore(); c.lineWidth = 3; c.strokeStyle = '#4a4446'; rr(c, cx - 70, cy - 66, 140, 180, 5); c.stroke(); }
    }
    c.restore();
  }
  function pastille(c, x, y, t) {
    c.save(); rr(c, x - 52, y - 17, 104, 34, 17); c.fillStyle = 'rgba(250,243,239,.96)'; c.shadowColor = 'rgba(70,45,35,.2)'; c.shadowBlur = 8; c.fill(); c.shadowColor = 'transparent';
    c.fillStyle = '#3a2a24'; c.font = '600 19px Nunito, Arial, sans-serif'; c.textAlign = 'center'; c.textBaseline = 'middle'; c.fillText(t, x, y + 1); c.restore();
  }

  /* ---- Modèles physiques distincts : chaque format a ses dimensions, sa silhouette, son anse/couvercle et sa zone imprimable.
         La photo de la cliente est une donnée indépendante : elle est projetée sur la surface cylindrique de chaque modèle. ---- */
  var MOD = {
    tasse: { H: 640, base: 556, ppm: 3.8, m: {
      '11oz': { nom: '11 oz', H: 95, D: 82, ph: 88, po: 5, ext: 34, th: 10, a: .14, b: .8 },
      '15oz': { nom: '15 oz', H: 119, D: 85, ph: 100, po: 12, ext: 40, th: 11.5, a: .12, b: .83 } } },
    isotherme: { H: 880, base: 800, ppm: 2.6, m: {
      '20oz': { nom: '20 oz', H: 190, D: 90, Db: 72, lid: 22, ph: 150, po: 16 },
      '30oz': { nom: '30 oz', H: 215, D: 97, Db: 78, lid: 24, ph: 172, po: 18 },
      '40oz': { nom: '40 oz', H: 250, D: 104, Db: 82, lid: 26, ph: 205, po: 20 } } }
  };
  var TM = 1.38;
  function ombreSol(c, cx, y, rx, ry) { c.save(); var g = c.createRadialGradient(cx, y, 1, cx, y, rx); g.addColorStop(0, 'rgba(50,30,20,.4)'); g.addColorStop(1, 'rgba(50,30,20,0)'); c.translate(0, y); c.scale(1, ry / rx); c.translate(0, -y); c.fillStyle = g; c.beginPath(); c.arc(cx, y, rx, 0, 7); c.fill(); c.restore(); }
  function textureCyl(im, tw, th, seulTexte) {
    var t = document.createElement('canvas'); t.width = tw; t.height = th; var c = t.getContext('2d'); c.fillStyle = '#fff'; c.fillRect(0, 0, tw, th);
    var K = 2.3; if (seulTexte) { c.save(); c.scale(K, K); ecrire(c, tw / 2 / K, th / 2 / K, tw * .75 / K); c.restore(); return t; }
    if (im) cover(c, im, 0, 0, tw, th);
    else { var g = c.createLinearGradient(0, 0, tw, th); g.addColorStop(0, '#f3e1d8'); g.addColorStop(1, '#dcb9aa'); c.fillStyle = g; c.fillRect(0, 0, tw, th); c.fillStyle = 'rgba(92,47,43,.55)'; c.textAlign = 'center'; c.textBaseline = 'middle'; c.font = '600 ' + Math.round(tw * .06) + 'px Nunito, Arial, sans-serif'; c.fillText('Votre photo ici', tw / 2, th / 2); }
    c.save(); c.scale(K, K); ecrire(c, tw / 2 / K, (th - Math.max(50, th * .12)) / K, tw * .72 / K); c.restore(); return t;
  }
  function tranches(c, T, R, ry, cx, y0, hh) {
    var tw = T.width, th = T.height;
    for (var x = -R + 1; x < R - 1; x += 1) {
      var u = x / R, a = Math.asin(u); if (Math.abs(a) > TM) continue;
      var sx = (a / TM + 1) / 2 * tw, sw = Math.max(2, tw / (2 * TM) / (R * Math.sqrt(1 - u * u)) * 1.2);
      c.drawImage(T, Math.min(tw - sw, Math.max(0, sx - sw / 2)), 0, sw, th, cx + x, y0 + ry * Math.sqrt(1 - u * u), 1.6, hh);
    }
  }
  function ombrage(c, cx, R, y0, y1, flip) {
    var g = c.createLinearGradient(cx - R, 0, cx + R, 0), s = function (p) { return flip ? 1 - p : p; };
    g.addColorStop(0, 'rgba(60,40,30,.34)'); g.addColorStop(.13, 'rgba(60,40,30,.04)'); g.addColorStop(s(.3), 'rgba(255,255,255,.22)'); g.addColorStop(s(.36), 'rgba(255,255,255,0)'); g.addColorStop(.8, 'rgba(60,40,30,.1)'); g.addColorStop(1, 'rgba(60,40,30,.42)');
    c.fillStyle = g; c.fillRect(cx - R, y0, R * 2, y1 - y0);
  }
  function dessinerTasse(c, M, ppm, cx, base, im, o) {
    var R = M.D / 2 * ppm, Ht = M.H * ppm, top = base - Ht, ry = R * .2, s = o.flip ? -1 : 1;
    ombreSol(c, cx + s * R * .25, base + ry * .35, R * 1.55, ry * 1.1);
    var hx = cx + s * (R - 4), y1 = top + Ht * M.a, y2 = top + Ht * M.b, ex = M.ext * ppm;
    function anse(w, col) { c.lineWidth = w; c.strokeStyle = col; c.lineCap = 'round'; c.beginPath(); c.moveTo(hx, y1); c.bezierCurveTo(hx + s * ex * 1.3, y1 - 6, hx + s * ex * 1.3, y2 + 6, hx, y2); c.stroke(); }
    c.save(); anse(M.th * ppm + 5, '#c9bfba'); anse(M.th * ppm, '#f1eeec'); anse(M.th * ppm * .34, '#ffffff'); c.restore();
    c.save(); c.beginPath(); c.moveTo(cx - R, top); c.lineTo(cx - R, base); c.ellipse(cx, base, R, ry, 0, Math.PI, 0, true); c.lineTo(cx + R, top); c.ellipse(cx, top, R, ry, 0, 0, Math.PI, true); c.closePath(); c.clip();
    var gb = c.createLinearGradient(cx - R, 0, cx + R, 0); gb.addColorStop(0, '#e4dfdc'); gb.addColorStop(.5, '#ffffff'); gb.addColorStop(1, '#e1dbd8'); c.fillStyle = gb; c.fillRect(cx - R, top - ry, R * 2, Ht + ry * 2);
    if (o.photo) { var hh = M.ph * ppm, tw = 900, T = textureCyl(im, tw, Math.round(tw * hh / (2 * TM * R)), o.photo === 'texte'); tranches(c, T, R, ry, cx, top + M.po * ppm, hh); }
    ombrage(c, cx, R, top - ry, base + ry, o.flip); c.restore();
    c.beginPath(); c.ellipse(cx, top, R, ry, 0, 0, 7); c.fillStyle = '#fbfaf9'; c.fill(); c.lineWidth = 1.5; c.strokeStyle = '#d6d0cc'; c.stroke();
    var gi = c.createLinearGradient(0, top - ry, 0, top + ry); gi.addColorStop(0, '#b9aba3'); gi.addColorStop(1, '#ece5e1');
    c.beginPath(); c.ellipse(cx, top + 1, R - 4.5, ry - 3.5, 0, 0, 7); c.fillStyle = gi; c.fill();
  }
  function dessinerVerre(c, M, ppm, cx, base, im, o) {
    var R = M.D / 2 * ppm, Rb = M.Db / 2 * ppm, Ht = M.H * ppm, top = base - Ht, ry = R * .13, ryb = Rb * .13, band = (M.H - M.po - M.ph) * ppm, yb = base - band, Rm = Rb + (R - Rb) * band / Ht;
    ombreSol(c, cx + 12, base + ryb * .3, R * 1.5, ryb * 1.2);
    c.save(); c.beginPath(); c.moveTo(cx - R, top); c.lineTo(cx - Rb, base); c.ellipse(cx, base, Rb, ryb, 0, Math.PI, 0, true); c.lineTo(cx + R, top); c.ellipse(cx, top, R, ry, 0, 0, Math.PI, true); c.closePath(); c.clip();
    var gb = c.createLinearGradient(cx - R, 0, cx + R, 0); gb.addColorStop(0, '#e6e2e0'); gb.addColorStop(.5, '#ffffff'); gb.addColorStop(1, '#e3dfdd'); c.fillStyle = gb; c.fillRect(cx - R, top - ry, R * 2, Ht + ry * 2);
    var yp = top + M.po * ppm, hh = M.ph * ppm;
    if (o.photo) {
      var tw = 900, T = textureCyl(im, tw, Math.round(tw * hh / (2 * TM * R)), o.photo === 'texte'); tranches(c, T, R, ry, cx, yp, hh);
      var gf = c.createLinearGradient(0, yp + hh * .5, 0, yp + hh + ry); gf.addColorStop(0, 'rgba(243,240,241,0)'); gf.addColorStop(1, 'rgba(243,240,241,1)'); c.fillStyle = gf; c.fillRect(cx - R, yp + hh * .5, R * 2, hh * .5 + ry * 2);
    }
    ombrage(c, cx, R, top - ry, base + ryb, o.flip);
    var gs = c.createLinearGradient(cx - R, 0, cx + R, 0); ['#f4f4f4', '#a9abad', '#eeeeef', '#8e9092', '#d4d5d6'].forEach(function (col, i) { gs.addColorStop(i / 4, col); });
    c.beginPath(); c.moveTo(cx - Rm, yb); c.ellipse(cx, yb, Rm, ryb * 1.05, 0, Math.PI, 0, true); c.lineTo(cx + Rb, base); c.ellipse(cx, base, Rb, ryb, 0, 0, Math.PI, false); c.closePath(); c.fillStyle = gs; c.fill();
    c.restore();
    c.beginPath(); c.ellipse(cx, top, R, ry, 0, 0, 7); c.fillStyle = '#e9e9ea'; c.fill(); c.lineWidth = 2; c.strokeStyle = '#b3b5b7'; c.stroke();
    // couvercle
    var lh = M.lid * ppm, lr = R * .98;
    c.save(); var gl = c.createLinearGradient(cx - lr, 0, cx + lr, 0); gl.addColorStop(0, 'rgba(150,155,160,.88)'); gl.addColorStop(.3, 'rgba(235,238,240,.78)'); gl.addColorStop(1, 'rgba(130,135,140,.88)');
    c.beginPath(); c.moveTo(cx - lr, top); c.lineTo(cx - lr * .93, top - lh); c.ellipse(cx, top - lh, lr * .93, ry * .9, 0, Math.PI, 0, false); c.lineTo(cx + lr, top); c.ellipse(cx, top, lr, ry, 0, 0, Math.PI, false); c.closePath(); c.fillStyle = gl; c.fill();
    c.beginPath(); c.ellipse(cx, top - lh, lr * .93, ry * .9, 0, 0, 7); c.fillStyle = 'rgba(245,247,248,.95)'; c.fill(); c.lineWidth = 1.5; c.strokeStyle = 'rgba(120,125,130,.8)'; c.stroke();
    c.beginPath(); c.ellipse(cx + lr * .3, top - lh, lr * .2, ry * .4, 0, 0, 7); c.fillStyle = 'rgba(90,95,100,.75)'; c.fill(); c.restore();
  }
  function fondStudio(c, W, H) { var g = c.createLinearGradient(0, 0, 0, H); g.addColorStop(0, '#f8f1ec'); g.addColorStop(1, '#ead9cf'); c.fillStyle = g; c.fillRect(0, 0, W, H); }
  function fondTable(c, W, H, yT) {
    var g = c.createLinearGradient(0, 0, 0, yT); g.addColorStop(0, '#5b3f30'); g.addColorStop(1, '#8a6247'); c.fillStyle = g; c.fillRect(0, 0, W, yT);
    [[90, 90, 52], [250, 170, 34], [420, 70, 40], [600, 140, 56], [760, 60, 36], [900, 150, 48], [980, 70, 26], [60, 250, 36], [330, 260, 30], [820, 270, 40]].forEach(function (p) { if (p[1] > yT - 20) return; var r = c.createRadialGradient(p[0], p[1], 0, p[0], p[1], p[2]); r.addColorStop(0, 'rgba(255,228,165,.9)'); r.addColorStop(.7, 'rgba(255,205,125,.35)'); r.addColorStop(1, 'rgba(255,200,120,0)'); c.fillStyle = r; c.beginPath(); c.arc(p[0], p[1], p[2], 0, 7); c.fill(); });
    c.save(); c.beginPath(); c.moveTo(640, yT + 10); c.quadraticCurveTo(660, yT - 150, 790, yT - 160); c.quadraticCurveTo(960, yT - 170, 1000, yT - 90); c.lineTo(1000, yT + 10); c.closePath(); var kg = c.createLinearGradient(0, yT - 160, 0, yT); kg.addColorStop(0, '#e6d7c4'); kg.addColorStop(1, '#c8b49a'); c.fillStyle = kg; c.fill(); c.clip();
    c.strokeStyle = 'rgba(120,95,70,.25)'; c.lineWidth = 3; for (var x = 640; x < 1010; x += 12) { c.beginPath(); c.moveTo(x, yT - 170); c.lineTo(x - 8, yT + 12); c.stroke(); } c.restore();
    var w = c.createLinearGradient(0, yT, 0, H); w.addColorStop(0, '#b98b5e'); w.addColorStop(1, '#7c5636'); c.fillStyle = w; c.fillRect(0, yT, W, H - yT);
    for (var i = 0; i < 26; i++) { c.strokeStyle = 'rgba(70,40,20,' + (.08 + (i % 4) * .04) + ')'; c.lineWidth = 1 + (i % 3); var yy = yT + 6 + i * (H - yT) / 26; c.beginPath(); c.moveTo(0, yy); c.bezierCurveTo(W * .3, yy + (i % 5) - 2, W * .6, yy - (i % 4), W, yy + 2); c.stroke(); }
  }
  function boiteCadeau(c, cx, base, bw, bh, avant) {
    if (!avant) { c.fillStyle = '#b48a5c'; c.fillRect(cx - bw / 2, base - bh, bw, bh); c.strokeStyle = 'rgba(245,230,205,.8)'; c.lineWidth = 3; for (var i = 0; i < 70; i++) { var x = cx - bw / 2 + 8 + (i * 37) % (bw - 16), y = base - bh + 4 + (i * 13) % 40; c.beginPath(); c.moveTo(x, y); c.lineTo(x + 18, y - 9 + (i % 3) * 6); c.stroke(); } return; }
    var g = c.createLinearGradient(cx - bw / 2, 0, cx + bw / 2, 0); g.addColorStop(0, '#a87b4d'); g.addColorStop(.5, '#c79b6a'); g.addColorStop(1, '#a07446'); c.fillStyle = g; c.fillRect(cx - bw / 2, base - bh * .62, bw, bh * .62 + 6);
    c.fillStyle = 'rgba(40,20,10,.18)'; c.fillRect(cx - bw / 2, base - bh * .62, bw, 6);
    c.fillStyle = '#efe4d4'; c.fillRect(cx - 18, base - bh * .62, 36, bh * .62 + 6);
    var bx = cx, by = base - bh * .62; c.fillStyle = '#efe4d4'; c.strokeStyle = '#d9c9b2'; c.lineWidth = 2;
    [[-1, 0], [1, 0]].forEach(function (d) { c.save(); c.translate(bx, by); c.scale(d[0], 1); c.beginPath(); c.moveTo(0, 0); c.bezierCurveTo(30, -62, 105, -66, 96, -8); c.bezierCurveTo(88, 22, 30, 12, 0, 0); c.fill(); c.stroke(); c.restore(); });
    c.beginPath(); c.arc(bx, by, 15, 0, 7); c.fill(); c.stroke();
  }
  function cylindre(cv, cle, im, o) {
    var K = MOD[cle], v = (o.vue || {}).id || 'face', W = 1000, H = K.H, k = (o.w || W) / W, ppm = K.ppm;
    cv.width = Math.round(W * k); cv.height = Math.round(H * k);
    var c = cv.getContext('2d'); c.imageSmoothingQuality = 'high'; c.scale(k, k);
    var cle2 = variante(), M = K.m[cle2] || K.m[Object.keys(K.m)[0]], base = K.base, tasse = cle === 'tasse';
    var draw = function (m, x, y, op) { (tasse ? dessinerTasse : dessinerVerre)(c, m, ppm, x, y, im, op); };
    var dosTxt = (txt.t && txt.pos === 'derriere') ? 'texte' : null;
    if (v === 'face') { fondStudio(c, W, H); draw(M, tasse ? 470 : 500, base, { photo: 'photo' }); }
    else if (v === 'dos') { fondStudio(c, W, H); draw(M, tasse ? 530 : 500, base, { flip: true, photo: dosTxt }); }
    else if (v === 'table') { var yT = base - 36; fondTable(c, W, H, yT); draw(M, tasse ? 420 : 400, base, { photo: 'photo' }); }
    else if (v === 'cadeau') {
      fondStudio(c, W, H); var bw = (M.D * ppm) * 1.7, bh = (tasse ? M.H * .62 : M.H * .34) * ppm, cx = 500;
      boiteCadeau(c, cx, base + 30, bw, bh, false); draw(M, cx - (tasse ? 14 : 0), base + 4, { photo: 'photo' }); boiteCadeau(c, cx, base + 30, bw, bh, true);
    } else {
      fondStudio(c, W, H); var ks = Object.keys(K.m).filter(function (x) { return P.v[x]; }), n = ks.length;
      ks.forEach(function (x, i) { var cx = tasse ? [185, 661][i] : W * (i + .5) / n; draw(K.m[x], cx, base - 20, { photo: 'photo' }); pastille(c, cx, base + 30, K.m[x].nom); });
    }
  }

  function dessiner(cv, cle, im, o) {
    o = o || {};
    var b = BASES[cle]; if (!b || !b.pret) return;
    if (b.cyl) return cylindre(cv, cle, im, o);
    var v = o.vue || {}, ech = o.ech || 1, W = b.W, H = b.H, k = (o.w || W) / W;
    cv.width = Math.round(W * k); cv.height = Math.round(H * k);
    var c = cv.getContext('2d'); c.imageSmoothingQuality = 'high';
    function vueT() {
      c.scale(k, k);
      var px = W / 2, py = H * .78; c.translate(px, py); c.scale(ech, ech); c.translate(-px, -py);
      if (v.zoom) { var z = v.zoom; c.translate(W / 2, H / 2); c.scale(z[2], z[2]); c.translate(-z[0], -z[1]); }
    }
    c.save(); vueT();
    if (cle === 'proc') {
      var fond = c.createLinearGradient(0, 0, 0, H); fond.addColorStop(0, '#f6eee9'); fond.addColorStop(1, '#ead9cf'); c.fillStyle = fond; c.fillRect(-400, -400, W + 800, H + 800);
      var vr = variante(); objet(c, vr, im, 228, 335, true); objet(c, vr, im, 472, 335, false); pastille(c, 228, 520, 'Avant'); pastille(c, 472, 520, 'Arrière');
      c.restore(); return;
    }
    if (v.miroir) { c.translate(W, 0); c.scale(-1, 1); }
    c.drawImage(b.img, 0, 0);
    c.restore();
    if (!im) return;
    c.save(); vueT();
    if (b.zone) {
      var z = b.zone, zx = v.miroir ? W - z.x - z.w : z.x, m = v.miroir ? -1 : 1;
      c.save(); rr(c, zx, z.y, z.w, z.h, z.r || 0); c.clip();
      var dos = v.miroir && txt.pos === 'derriere' && txt.t && (CODE === 'tasse' || CODE === 'isotherme');
      if (dos) { c.fillStyle = '#f4f1f2'; c.fillRect(zx, z.y, z.w, z.h); }
      else if (z.fondu) {
        cover(c, im, zx - 1, z.y - 1, z.w + 2, z.fondu[1] - z.y + 40);
        var g0 = c.createLinearGradient(0, z.fondu[0], 0, z.fondu[1] + 18); g0.addColorStop(0, 'rgba(243,240,241,0)'); g0.addColorStop(1, 'rgba(243,240,241,1)');
        c.fillStyle = g0; c.fillRect(zx, z.fondu[0], z.w, z.h); c.fillStyle = '#f3f0f1'; c.fillRect(zx, z.fondu[1] + 17, z.w, z.h);
      } else cover(c, im, zx - 1, z.y - 1, z.w + 2, z.h + 2);
      if (txt.t && ((!v.miroir && txt.pos === 'devant') || dos)) ecrire(c, zx + z.w / 2, z.y + z.h - (z.fondu ? 62 : 34), z.w - 28);
      var g = c.createLinearGradient(zx, 0, zx + z.w, 0), s = function (p) { return m > 0 ? p : 1 - p; };
      g.addColorStop(0, 'rgba(40,25,20,.34)'); g.addColorStop(.16, 'rgba(40,25,20,0)'); g.addColorStop(s(.62), 'rgba(255,255,255,.07)'); g.addColorStop(.86, 'rgba(40,25,20,0)'); g.addColorStop(1, 'rgba(40,25,20,.4)');
      c.fillStyle = g; c.fillRect(zx, z.y, z.w, z.h);
      var vg = c.createLinearGradient(0, z.y, 0, z.y + z.h); vg.addColorStop(0, 'rgba(40,25,20,.12)'); vg.addColorStop(.1, 'rgba(40,25,20,0)'); vg.addColorStop(.92, 'rgba(40,25,20,0)'); vg.addColorStop(1, 'rgba(40,25,20,.16)');
      c.fillStyle = vg; c.fillRect(zx, z.y, z.w, z.h); c.restore();
    } else if (b.rond) {
      var q = b.rond; c.save(); c.beginPath(); c.arc(q.x, q.y, q.r, 0, 7); c.clip(); cover(c, im, q.x - q.r, q.y - q.r, q.r * 2, q.r * 2); ecrire(c, q.x, q.y + q.r - 30, q.r * 1.45);
      var rg = c.createRadialGradient(q.x - q.r * .35, q.y - q.r * .45, q.r * .05, q.x, q.y, q.r);
      rg.addColorStop(0, 'rgba(255,255,255,.3)'); rg.addColorStop(.35, 'rgba(255,255,255,.04)'); rg.addColorStop(.85, 'rgba(255,255,255,0)'); rg.addColorStop(1, 'rgba(255,255,255,.22)');
      c.fillStyle = rg; c.fillRect(q.x - q.r, q.y - q.r, q.r * 2, q.r * 2); c.restore();
    }
    c.restore();
  }

  /* ---- galerie : vue principale + vignettes dessinées avec la photo de la cliente ---- */
  var vitrine = $('pp-vues');
  function echelle() { return 1; }
  function dessinerVues() {
    var cle = cleBase(variante()), b = BASES[cle]; if (!b || !b.pret) return;
    var n = b.vues.length; if (vue >= n) vue = 0;
    dessiner(cvA, cle, img, { vue: b.vues[vue], ech: echelle(), w: 1000 });
    if (vitrine) {
      while (vitrine.children.length < n) { (function (i) { var bt = document.createElement('button'); bt.type = 'button'; bt.className = 'vig'; bt.setAttribute('aria-label', 'Vue ' + (i + 1)); bt.appendChild(document.createElement('canvas')); bt.addEventListener('click', function () { vue = i; maj(); }); vitrine.appendChild(bt); })(vitrine.children.length); }
      while (vitrine.children.length > n) vitrine.removeChild(vitrine.lastChild);
      Array.prototype.forEach.call(vitrine.children, function (bt, i) { bt.classList.toggle('on', i === vue); dessiner(bt.firstChild, cle, img, { vue: b.vues[i], ech: echelle(), w: 280 }); });
    }
  }
  Array.prototype.forEach.call(document.querySelectorAll('[data-vue]'), function (bt) {
    bt.addEventListener('click', function () {
      var cle = cleBase(variante()), n = BASES[cle].vues.length, d = bt.getAttribute('data-vue');
      vue = d === 'zoom' ? n - 1 : (vue + Number(d) + n) % n; maj();
    });
  });
  Array.prototype.forEach.call(document.querySelectorAll('[data-modele]'), function (bt) {
    bt.addEventListener('click', function () { var r = f.querySelector('input[name=variante][value="' + bt.getAttribute('data-modele') + '"]'); if (r) { r.checked = true; maj(); } });
  });

  /* ---- prix ---- */
  function fmt(n) { return (Math.round(n * 100) / 100).toLocaleString('fr-CA', { minimumFractionDigits: 2 }) + ' $'; }
  function maj() {
    dessinerVues();
    var v = P.v[variante()] || P.v[Object.keys(P.v)[0]], q = Math.max(1, Math.min(10, Number(qte.value) || 1));
    $('pp-prix').textContent = fmt(v[1]);
    $('pp-total-prod').textContent = fmt(v[1] * q);
    var badge = $('pp-badge-txt'); if (badge && CODE === 'tasse') badge.innerHTML = '<b>' + v[0].split(' (')[0] + '</b><small>(' + (v[0].split(' (')[1] || '').replace(')', '') + ')</small>';
    Array.prototype.forEach.call(document.querySelectorAll('[data-modele]'), function (bt) { bt.classList.toggle('on', bt.getAttribute('data-modele') === variante()); });
  }
  window.__maj = maj;
  Array.prototype.forEach.call(radios(), function (r) { r.addEventListener('change', maj); });
  Array.prototype.forEach.call(document.querySelectorAll('[data-qte]'), function (b) { b.addEventListener('click', function () { qte.value = Math.max(1, Math.min(10, (Number(qte.value) || 1) + Number(b.getAttribute('data-qte')))); maj(); }); });
  qte.addEventListener('input', maj);
  var liste = $('pp-liste'); if (liste) liste.addEventListener('click', function () { liste.classList.toggle('on'); });

  /* ---- texte : police, couleur, devant / derrière ---- */
  var tIn = $('pp-texte'), tOpts = $('pp-texte-opts');
  function chargerPolice() { if (document.fonts && document.fonts.load) document.fonts.load('40px "' + txt.police + '"').then(function () { maj(); }); }
  tIn.addEventListener('input', function () { txt.t = this.value; $('pp-compte').textContent = this.value.length + ' / 50'; tOpts.hidden = !this.value.trim(); chargerPolice(); maj(); });
  Array.prototype.forEach.call(f.querySelectorAll('[data-police]'), function (b) { b.addEventListener('click', function () { txt.police = b.getAttribute('data-police'); Array.prototype.forEach.call(f.querySelectorAll('[data-police]'), function (x) { x.classList.toggle('on', x === b); }); chargerPolice(); maj(); }); });
  Array.prototype.forEach.call(f.querySelectorAll('[data-couleur]'), function (b) { b.addEventListener('click', function () { txt.couleur = b.getAttribute('data-couleur'); Array.prototype.forEach.call(f.querySelectorAll('[data-couleur]'), function (x) { x.classList.toggle('on', x === b); }); maj(); }); });
  Array.prototype.forEach.call(f.querySelectorAll('input[name=position]'), function (r) { r.addEventListener('change', function () { txt.pos = r.value; if (CODE === 'tasse' || CODE === 'isotherme') { vue = r.value === 'derriere' ? 1 : 0; } maj(); }); });
  Array.prototype.forEach.call(f.querySelectorAll('[data-couleur]'), function (b) { if (b.getAttribute('data-couleur') === txt.couleur) b.classList.add('on'); });

  /* ---- cadrage de la photo : zoom et déplacement ---- */
  var aj = $('pp-ajust'), zm = $('pp-zoom');
  function zoomer(z) { cad.z = Math.max(1, Math.min(3, z)); zm.value = cad.z; maj(); }
  zm.addEventListener('input', function () { cad.z = Number(zm.value) || 1; maj(); });
  $('pp-z-moins').addEventListener('click', function () { zoomer(cad.z - .15); });
  $('pp-z-plus').addEventListener('click', function () { zoomer(cad.z + .15); });
  $('pp-recentrer').addEventListener('click', function () { cad = { z: 1, x: 0, y: 0 }; zm.value = 1; maj(); });
  var glisse = null;
  cvA.addEventListener('pointerdown', function (e) { if (!img) return; glisse = { x: e.clientX, y: e.clientY }; try { cvA.setPointerCapture(e.pointerId); } catch (er) {} });
  cvA.addEventListener('pointermove', function (e) {
    if (!glisse) return;
    cad.x = Math.max(-1, Math.min(1, cad.x - (e.clientX - glisse.x) / (cvA.clientWidth * .22)));
    cad.y = Math.max(-1, Math.min(1, cad.y - (e.clientY - glisse.y) / (cvA.clientHeight * .28)));
    glisse = { x: e.clientX, y: e.clientY }; maj();
  });
  ['pointerup', 'pointercancel'].forEach(function (ev) { cvA.addEventListener(ev, function () { glisse = null; }); });
  cvA.addEventListener('wheel', function (e) { if (!img) return; e.preventDefault(); zoomer(cad.z + (e.deltaY < 0 ? .1 : -.1)); }, { passive: false });

  /* ---- téléversement ---- */
  var zone = $('pp-zone'), vign = $('pp-vignette'), alerte = $('pp-alerte'), src = null;
  $('pp-changer').addEventListener('click', function () { fi.click(); });
  ['dragover', 'dragenter'].forEach(function (ev) { zone.addEventListener(ev, function (e) { e.preventDefault(); zone.classList.add('survol'); }); });
  ['dragleave', 'drop'].forEach(function (ev) { zone.addEventListener(ev, function (e) { e.preventDefault(); zone.classList.remove('survol'); }); });
  zone.addEventListener('drop', function (e) { if (e.dataTransfer.files[0]) { fi.files = e.dataTransfer.files; lire(); } });
  fi.addEventListener('change', lire);
  function reduire(im, max, q) {
    var m = Math.min(1, max / Math.max(im.width, im.height)), cv = document.createElement('canvas'); cv.width = Math.round(im.width * m); cv.height = Math.round(im.height * m);
    cv.getContext('2d').drawImage(im, 0, 0, cv.width, cv.height); return cv.toDataURL('image/jpeg', q);
  }
  function afficherPhoto(im, nom, taille) {
    img = im; photoData = true; nomFichier = nom; tailleFichier = taille;
    $('pp-miniature').src = reduire(im, 200, .8); vign.hidden = false; zone.hidden = true; aj.hidden = false;
    $('pp-nom').textContent = nom; $('pp-taille').textContent = taille; maj();
  }
  function lire() {
    var fic = fi.files[0]; alerte.hidden = true;
    if (!fic) return;
    if (!/^image\/(jpeg|png)$/.test(fic.type)) { alerte.textContent = 'Choisissez une photo JPEG ou PNG.'; alerte.hidden = false; fi.value = ''; return; }
    if (fic.size > 10 * 1024 * 1024) { alerte.textContent = 'La photo est trop lourde (maximum 10 Mo).'; alerte.hidden = false; fi.value = ''; return; }
    var im = new Image(), url = URL.createObjectURL(fic);
    im.onload = function () {
      src = reduire(im, 1400, .86); cad = { z: 1, x: 0, y: 0 }; zm.value = 1;
      var t = fic.size > 1048576 ? (fic.size / 1048576).toFixed(1).replace('.', ',') + ' Mo' : Math.round(fic.size / 1024) + ' Ko';
      var im2 = new Image(); im2.onload = function () { afficherPhoto(im2, fic.name, t); }; im2.src = src;
      if (Math.max(im.width, im.height) < 900) { alerte.textContent = "Cette photo est de petite taille : l'impression pourrait manquer de netteté. Une photo plus grande donnerait un meilleur résultat."; alerte.hidden = false; }
    };
    im.onerror = function () { alerte.textContent = "Cette photo n'a pas pu être lue."; alerte.hidden = false; };
    im.src = url;
  }

  /* ---- ajouter au panier ---- */
  var ASPECT = { tasse: [267, 296], isotherme: [234, 394], rond: [1, 1], ornement: [1, 1], coeur: [228, 222], rect: [172, 234], cuir: [140, 180] };
  function exporter() {
    var a = ASPECT[CODE === 'portecles' ? variante() : CODE] || [1, 1], long = 1500, w = a[0] >= a[1] ? long : Math.round(long * a[0] / a[1]), h = a[0] >= a[1] ? Math.round(long * a[1] / a[0]) : long;
    var cv = document.createElement('canvas'); cv.width = w; cv.height = h; var c = cv.getContext('2d'); cover(c, img, 0, 0, w, h); return cv.toDataURL('image/jpeg', .88);
  }
  var q0 = new URLSearchParams(location.search).get('modifier');
  if (q0 && window.NBQ) {
    var ex = window.NBQ.lire().filter(function (x) { return x.id === q0; })[0];
    if (ex && ex.src) {
      idModif = ex.id; var r = f.querySelector('input[name=variante][value="' + ex.variante + '"]'); if (r) r.checked = true;
      qte.value = ex.quantite; cad = ex.cad || cad; zm.value = cad.z; txt.t = ex.texte || ''; txt.police = ex.police || txt.police; txt.couleur = ex.couleur || txt.couleur; txt.pos = ex.position || 'devant';
      tIn.value = txt.t; $('pp-compte').textContent = txt.t.length + ' / 50'; tOpts.hidden = !txt.t;
      Array.prototype.forEach.call(f.querySelectorAll('[data-police]'), function (x) { x.classList.toggle('on', x.getAttribute('data-police') === txt.police); });
      Array.prototype.forEach.call(f.querySelectorAll('[data-couleur]'), function (x) { x.classList.toggle('on', x.getAttribute('data-couleur') === txt.couleur); });
      var rp = f.querySelector('input[name=position][value="' + txt.pos + '"]'); if (rp) rp.checked = true;
      src = ex.src; var im3 = new Image(); im3.onload = function () { afficherPhoto(im3, ex.nom || 'photo.jpg', ex.taille || ''); chargerPolice(); }; im3.src = src;
      $('pp-ajouter').firstChild.nextSibling.textContent = 'Enregistrer les changements';
    }
  }
  $('pp-ajouter').addEventListener('click', function () {
    if (!img) { alerte.textContent = "Ajoutez d'abord votre photo."; alerte.hidden = false; zone.hidden ? vign.scrollIntoView({ block: 'center', behavior: 'smooth' }) : zone.scrollIntoView({ block: 'center', behavior: 'smooth' }); return; }
    var cle = cleBase(variante()), b = BASES[cle], keep = vue; vue = 0;
    var ap = document.createElement('canvas'); dessiner(ap, cle, img, { vue: b.vues[0], ech: echelle(), w: 360 }); vue = keep;
    var item = { id: idModif || ('i' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6)), code: CODE, variante: variante(), quantite: Number(qte.value) || 1,
      texte: txt.t.trim(), police: txt.t.trim() ? txt.police : '', couleur: txt.t.trim() ? txt.couleur : '', position: txt.t.trim() ? txt.pos : '', cad: cad, cadrage: JSON.stringify(cad),
      photo: exporter(), src: src, nom: nomFichier, taille: tailleFichier, apercu: ap.toDataURL('image/jpeg', .8) };
    var a = window.NBQ.lire(), i = a.map(function (x) { return x.id; }).indexOf(item.id);
    if (i > -1) a[i] = item; else a.push(item);
    if (!window.NBQ.ecrire(a)) { alerte.textContent = "Le panier est plein : finalisez ou retirez un article, puis réessayez."; alerte.hidden = false; return; }
    var ok = $('pp-ajoute'); ok.hidden = false; ok.scrollIntoView({ block: 'center', behavior: 'smooth' });
  });
  maj();
})();

/* ---------- Page panier ---------- */
(function () {
  var $ = function (id) { return document.getElementById(id); };
  var f = $('form-panier'); if (!f || !window.NBQ) return;
  var C = window.NORELIAS || {}, PR = JSON.parse(f.getAttribute('data-produits')), LIV = Number(f.getAttribute('data-livraison'));
  var racine = (document.querySelector('link[href*="style.css"]').getAttribute('href') || '').replace(/style\.css.*$/, '');
  var TX = { QC: { tps: 5, tvq: 9.975 }, NB: { tvh: 15 }, NL: { tvh: 15 }, NS: { tvh: 14 }, PE: { tvh: 15 }, ON: { tvh: 13 }, AB: { tps: 5 }, BC: { tps: 5 }, MB: { tps: 5 }, SK: { tps: 5 }, NT: { tps: 5 }, NU: { tps: 5 }, YT: { tps: 5 } };
  var PREM = { A: 'NL', B: 'NS', C: 'PE', E: 'NB', G: 'QC', H: 'QC', J: 'QC', K: 'ON', L: 'ON', M: 'ON', N: 'ON', P: 'ON', R: 'MB', S: 'SK', T: 'AB', V: 'BC', X: 'NT', Y: 'YT' };
  var NOMS = { 'quebec': 'QC', 'new brunswick': 'NB', 'nouveau-brunswick': 'NB', 'nova scotia': 'NS', 'nouvelle-ecosse': 'NS', 'ontario': 'ON', 'manitoba': 'MB', 'saskatchewan': 'SK', 'alberta': 'AB', 'british columbia': 'BC', 'colombie-britannique': 'BC', 'prince edward island': 'PE', 'ile-du-prince-edouard': 'PE', 'newfoundland and labrador': 'NL', 'terre-neuve-et-labrador': 'NL', 'newfoundland': 'NL', 'yukon': 'YT', 'northwest territories': 'NT', 'territoires du nord-ouest': 'NT', 'nunavut': 'NU' };
  function provDe(nom) { var s = String(nom || '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim(); return NOMS[s] || (TX[s.toUpperCase()] ? s.toUpperCase() : ''); }
  function cpOk(prov, cp) { var c = String(cp || '').toUpperCase().replace(/\s/g, ''); if (!/^[A-Z]\d[A-Z]\d[A-Z]\d$/.test(c)) return false; var p = PREM[c[0]]; return p === prov || (p === 'NT' && prov === 'NU'); }
  function fmt(n) { return (Math.round(n * 100) / 100).toLocaleString('fr-CA', { minimumFractionDigits: 2 }) + ' $'; }
  function esc(t) { return String(t == null ? '' : t).replace(/[&<>"]/g, function (c) { return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]; }); }
  function provTaxes() { return (f.cadeau.value === 'oui' && $('d-adresse').value.trim()) ? $('d-prov').value : $('b-prov').value; }

  function rendre() {
    var a = window.NBQ.lire(), L = $('pn-liste');
    $('pn-vide').hidden = a.length > 0; $('pn-coord').hidden = !a.length; $('pn-resume').hidden = !a.length;
    L.innerHTML = a.map(function (x, i) {
      var p = PR[x.code] || { nom: x.code, v: {} }, v = p.v[x.variante] || ['', 0];
      return '<article class="pn-art" data-i="' + i + '"><img src="' + esc(x.apercu) + '" alt=""><div class="pn-inf"><b>' + esc(p.nom) + '</b><span>' + esc(v[0]) + '</span>' +
        (x.texte ? '<small>Texte : « ' + esc(x.texte) + ' »</small>' : '') +
        '<div class="pn-act"><a class="lien" href="' + racine + 'boutique/' + esc(x.code) + '/?modifier=' + esc(x.id) + '">Modifier</a><button type="button" class="lien" data-suppr="' + i + '">Retirer</button></div></div>' +
        '<div class="pn-qte"><button type="button" data-q="-1" aria-label="Moins">&minus;</button><b>' + (Number(x.quantite) || 1) + '</b><button type="button" data-q="1" aria-label="Plus">+</button></div>' +
        '<div class="pn-prix">' + fmt(v[1] * (Number(x.quantite) || 1)) + '</div></article>';
    }).join('');
    totaux();
  }
  function totaux() {
    var a = window.NBQ.lire(), sous = 0;
    a.forEach(function (x) { var v = (PR[x.code] || { v: {} }).v[x.variante]; if (v) sous += v[1] * (Number(x.quantite) || 1); });
    var base = sous + LIV, prov = provTaxes(), t = TX[prov], tot = base, n = a.reduce(function (s, x) { return s + (Number(x.quantite) || 1); }, 0);
    var lig = '<dt>Sous-total (' + n + ' article' + (n > 1 ? 's' : '') + ')</dt><dd>' + fmt(sous) + '</dd><dt>Livraison par la poste</dt><dd>' + fmt(LIV) + '</dd>';
    function ajout(nom, p) { var m = Math.round(base * p) / 100; tot += m; lig += '<dt>' + nom + '</dt><dd>' + fmt(m) + '</dd>'; }
    if (t) { if (t.tps) ajout('TPS (5 %)', t.tps); if (t.tvq) ajout('TVQ (9,975 %)', t.tvq); if (t.tvh) ajout('TVH (' + t.tvh + ' %)', t.tvh); }
    else lig += '<dt class="gris">Taxes</dt><dd class="gris">selon votre province</dd>';
    tot = Math.round(tot * 100) / 100;
    $('pp-totaux').innerHTML = lig + '<dt class="t">Total</dt><dd class="t">' + fmt(tot) + (t ? '' : ' + taxes') + '</dd>';
    $('pp-btn-total').textContent = fmt(tot) + (t ? '' : ' + taxes');
  }
  $('pn-liste').addEventListener('click', function (e) {
    var b = e.target.closest('button'); if (!b) return; var a = window.NBQ.lire(), art = e.target.closest('.pn-art'), i = Number(art.getAttribute('data-i'));
    if (b.hasAttribute('data-suppr')) a.splice(i, 1);
    else if (b.hasAttribute('data-q')) a[i].quantite = Math.max(1, Math.min(10, (Number(a[i].quantite) || 1) + Number(b.getAttribute('data-q'))));
    else return;
    window.NBQ.ecrire(a); rendre();
  });
  Array.prototype.forEach.call(f.querySelectorAll('input[name=cadeau]'), function (r) { r.addEventListener('change', function () { $('b-dest').hidden = f.cadeau.value !== 'oui'; totaux(); }); });
  var m0 = $('msg-boutique'); if (m0 && /annule=1/.test(location.search)) { m0.textContent = "Le paiement a été annulé. Votre panier est conservé."; m0.hidden = false; }

  function adresse(p) {
    var rue = $(p + '-adresse'), ville = $(p + '-ville'), prov = $(p + '-prov'), cp = $(p + '-cp'), ul = $(p + '-liste'), items = [], cur = -1, t, ctl;
    function fermer() { ul.hidden = true; ul.innerHTML = ''; items = []; cur = -1; rue.setAttribute('aria-expanded', 'false'); }
    function choisir(i) { var a = items[i]; if (!a) return; rue.value = a.rue; ville.value = a.ville; if (a.prov) prov.value = a.prov; cp.value = a.cp; fermer(); totaux(); (a.cp ? cp : ville).focus(); }
    function montrer() {
      ul.innerHTML = ''; items.forEach(function (a, i) { var li = document.createElement('li'); li.setAttribute('role', 'option'); li.innerHTML = '<b></b><span></span>'; li.firstChild.textContent = a.rue; li.lastChild.textContent = a.ville + (a.prov ? ', ' + a.prov : '') + (a.cp ? ' ' + a.cp : ''); li.addEventListener('mousedown', function (e) { e.preventDefault(); choisir(i); }); ul.appendChild(li); });
      ul.hidden = !items.length; rue.setAttribute('aria-expanded', String(!!items.length)); cur = -1;
    }
    rue.addEventListener('input', function () {
      clearTimeout(t); var q = rue.value.trim(); if (q.length < 4) return fermer();
      t = setTimeout(function () {
        if (ctl && ctl.abort) ctl.abort(); ctl = window.AbortController ? new AbortController() : null;
        fetch('https://photon.komoot.io/api/?limit=10&lang=fr&bbox=-141.1,41.6,-52.5,83.2&q=' + encodeURIComponent(q), ctl ? { signal: ctl.signal } : {})
          .then(function (r) { return r.json(); })
          .then(function (j) {
            var vus = {}; items = [];
            (j.features || []).forEach(function (ft) {
              var pr = ft.properties || {}; if (pr.countrycode !== 'CA') return;
              var r = ((pr.housenumber ? pr.housenumber + ' ' : '') + (pr.street || '')).trim(); if (!r) return;
              var a = { rue: r, ville: pr.city || pr.town || pr.village || pr.municipality || pr.locality || pr.district || pr.county || '', prov: provDe(pr.state), cp: (pr.postcode || '').toUpperCase() };
              var cle = a.rue + '|' + a.ville + '|' + a.prov; if (vus[cle] || !a.prov) return; vus[cle] = 1; items.push(a);
            });
            montrer();
          }).catch(function () {});
      }, 320);
    });
    rue.addEventListener('keydown', function (e) {
      if (ul.hidden) return; var li = ul.children;
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); cur = (cur + (e.key === 'ArrowDown' ? 1 : -1) + li.length) % li.length; Array.prototype.forEach.call(li, function (x, i) { x.classList.toggle('on', i === cur); }); }
      else if (e.key === 'Enter' && cur > -1) { e.preventDefault(); choisir(cur); }
      else if (e.key === 'Escape') fermer();
    });
    rue.addEventListener('blur', function () { setTimeout(fermer, 150); });
    prov.addEventListener('change', totaux);
    cp.addEventListener('input', function () {
      var v = cp.value.toUpperCase().replace(/[^A-Z0-9 ]/g, ''); if (v.replace(' ', '').length > 3 && v.indexOf(' ') < 0) v = v.slice(0, 3) + ' ' + v.slice(3); cp.value = v.slice(0, 7);
      if (!prov.value && PREM[v[0]]) { prov.value = PREM[v[0]]; totaux(); }
    });
  }
  adresse('b'); adresse('d');

  function erreur(t) { var e = $('b-erreur'); e.textContent = t; e.hidden = !t; if (t) e.scrollIntoView({ block: 'center', behavior: 'smooth' }); }
  f.addEventListener('submit', function (e) {
    e.preventDefault(); erreur('');
    var a = window.NBQ.lire(); if (!a.length) return erreur('Votre panier est vide.');
    var d = {}; ['prenom', 'nom', 'courriel', 'telephone', 'adresse', 'ville', 'province', 'code_postal'].forEach(function (k) { d[k] = (f[k].value || '').trim(); });
    if (!d.prenom || !d.nom || !d.courriel || !d.telephone || !d.adresse || !d.ville || !d.province) return erreur('Remplissez tous les champs obligatoires (*).');
    if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(d.courriel)) return erreur('Courriel invalide.');
    if (!cpOk(d.province, d.code_postal)) return erreur('Vérifiez votre code postal : il doit correspondre à la province (ex. : E3N 2S1).');
    d.action = 'commander'; d.piege = f.piege.value; d.cadeau = f.cadeau.value === 'oui'; d.dest_nom = f.dest_nom.value; d.note = f.note.value;
    d.lignes = a.map(function (x) { return { produit: x.code, variante: x.variante, quantite: x.quantite, texte: x.texte, police: x.police, couleur: x.couleur, position: x.position, cadrage: x.cadrage, photo: x.photo }; });
    if (d.cadeau && f.dest_adresse.value.trim()) {
      d.dest_adresse = f.dest_adresse.value.trim(); d.dest_ville = f.dest_ville.value.trim(); d.dest_province = f.dest_province.value; d.dest_code_postal = f.dest_code_postal.value.trim();
      if (!d.dest_ville || !d.dest_province) return erreur("Complétez l'adresse du destinataire (ville et province).");
      if (d.dest_code_postal && !cpOk(d.dest_province, d.dest_code_postal)) return erreur('Le code postal du destinataire ne correspond pas à sa province.');
    }
    var b = $('b-envoyer'); b.disabled = true; b.firstChild.textContent = 'Un instant… ';
    fetch((C.serveur || '').replace('norelias-reservation', 'norelias-stripe'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(d) })
      .then(function (r) { return r.json(); })
      .then(function (j) { if (j && j.ok && j.paiement) { location.href = j.paiement; } else { throw new Error(j && j.erreur); } })
      .catch(function (er) { erreur((er && er.message) || 'Une erreur est survenue. Écrivez-moi à bonjour@norelias.ca.'); b.disabled = false; b.firstChild.textContent = 'Payer par carte · '; });
  });
  rendre();
})();
