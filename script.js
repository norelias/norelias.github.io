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

/* ---------- Boutique : aperçu, photo, commande ---------- */
(function () {
  var f = document.getElementById('form-boutique');
  var cartes = document.querySelectorAll('.prod[data-code]');
  if (!f) return;
  var C = window.NORELIAS || {};
  var P = JSON.parse(f.getAttribute('data-produits'));
  var LIV = Number(f.getAttribute('data-livraison'));
  var $ = function (id) { return document.getElementById(id); };
  var img = null, photoData = null;

  function cover(ctx, im, x, y, w, h) {
    var r = Math.max(w / im.width, h / im.height), sw = w / r, sh = h / r;
    ctx.drawImage(im, (im.width - sw) / 2, (im.height - sh) / 2, sw, sh, x, y, w, h);
  }
  function rr(ctx, x, y, w, h, r) { ctx.beginPath(); ctx.moveTo(x + r, y); ctx.arcTo(x + w, y, x + w, y + h, r); ctx.arcTo(x + w, y + h, x, y + h, r); ctx.arcTo(x, y + h, x, y, r); ctx.arcTo(x, y, x + w, y, r); ctx.closePath(); }
  // Aperçu réaliste : la photo de la cliente est placée sur une vraie photo du produit
  var BASES = {
    tasse: { src: 'images/boutique-tasse.jpg', zone: { x: 131, y: 46, w: 125, h: 135 }, ombre: true },
    isotherme: { src: 'images/boutique-isotherme.jpg', zone: { x: 141, y: 50, w: 100, h: 138 }, ombre: true },
    portecles: { src: 'images/boutique-portecles.jpg', quad: [[142, 68], [238, 85], [108, 172]], poly: [[143, 70], [236, 86], [219, 198], [110, 172]] },
    ornement: { src: 'images/boutique-ornement.jpg', rond: { x: 198, y: 111, r: 63 } }
  };
  var racine = (document.querySelector('link[href*="style.css"]').getAttribute('href') || '').replace(/style\.css.*$/, '');
  Object.keys(BASES).forEach(function (k) { var b = BASES[k]; b.img = new Image(); b.img.onload = function () { b.pret = true; if (window.__maj) window.__maj(); }; b.img.src = racine + b.src; });
  function dessiner(cv, code, im) {
    var b = BASES[code] || BASES.tasse; if (!b.pret) return;
    var T = 4, bw = b.img.width - 2 * T, bh = b.img.height - 2 * T, k = 1.6;
    cv.width = Math.round(bw * k); cv.height = Math.round(bh * k);
    var c = cv.getContext('2d'); c.imageSmoothingQuality = 'high';
    c.drawImage(b.img, T, T, bw, bh, 0, 0, cv.width, cv.height);
    if (!im) return;
    c.save(); c.scale(k, k); c.translate(-T, -T);
    if (b.zone) {
      var z = b.zone; c.beginPath(); c.rect(z.x, z.y, z.w, z.h); c.clip(); cover(c, im, z.x - 1, z.y - 1, z.w + 2, z.h + 2);
      var g = c.createLinearGradient(z.x, 0, z.x + z.w, 0);
      g.addColorStop(0, 'rgba(40,25,20,.38)'); g.addColorStop(.18, 'rgba(40,25,20,0)'); g.addColorStop(.62, 'rgba(255,255,255,.07)'); g.addColorStop(.85, 'rgba(40,25,20,0)'); g.addColorStop(1, 'rgba(40,25,20,.42)');
      c.fillStyle = g; c.fillRect(z.x, z.y, z.w, z.h);
      var v = c.createLinearGradient(0, z.y, 0, z.y + z.h); v.addColorStop(0, 'rgba(40,25,20,.12)'); v.addColorStop(.12, 'rgba(40,25,20,0)'); v.addColorStop(.9, 'rgba(40,25,20,0)'); v.addColorStop(1, 'rgba(40,25,20,.18)');
      c.fillStyle = v; c.fillRect(z.x, z.y, z.w, z.h);
    } else if (b.quad) {
      var q = b.quad, P = b.poly;
      c.beginPath(); c.moveTo(P[0][0], P[0][1]); for (var i = 1; i < P.length; i++) c.lineTo(P[i][0], P[i][1]); c.closePath(); c.clip();
      var W = Math.hypot(q[1][0] - q[0][0], q[1][1] - q[0][1]) * 1.1, H = Math.hypot(q[2][0] - q[0][0], q[2][1] - q[0][1]) * 1.1;
      var r = Math.max(W / im.width, H / im.height), sw = W / r, sh = H / r, sx = (im.width - sw) / 2, sy = (im.height - sh) / 2;
      var ux = (q[1][0] - q[0][0]) / (W / 1.1), uy = (q[1][1] - q[0][1]) / (W / 1.1), vx = (q[2][0] - q[0][0]) / (H / 1.1), vy = (q[2][1] - q[0][1]) / (H / 1.1);
      var ox = q[0][0] - ux * W * .05 - vx * H * .05, oy = q[0][1] - uy * W * .05 - vy * H * .05;
      c.transform(ux * W / sw, uy * W / sw, vx * H / sh, vy * H / sh, ox, oy); c.drawImage(im, sx, sy, sw, sh, 0, 0, sw, sh);
    } else if (b.rond) {
      var o = b.rond; c.beginPath(); c.arc(o.x, o.y, o.r, 0, 7); c.clip(); cover(c, im, o.x - o.r, o.y - o.r, o.r * 2, o.r * 2);
      var rg = c.createRadialGradient(o.x - o.r * .35, o.y - o.r * .45, o.r * .05, o.x, o.y, o.r);
      rg.addColorStop(0, 'rgba(255,255,255,.35)'); rg.addColorStop(.35, 'rgba(255,255,255,.05)'); rg.addColorStop(.85, 'rgba(255,255,255,0)'); rg.addColorStop(1, 'rgba(255,255,255,.28)');
      c.fillStyle = rg; c.fillRect(o.x - o.r, o.y - o.r, o.r * 2, o.r * 2);
    }
    c.restore();
  }
  // vignettes des cartes (photo d'exemple dessinée en dégradé)
  function exemple() {
    var cv = document.createElement('canvas'); cv.width = 400; cv.height = 400; var c = cv.getContext('2d');
    var g = c.createLinearGradient(0, 0, 400, 400); g.addColorStop(0, '#f0c9bd'); g.addColorStop(1, '#a98374'); c.fillStyle = g; c.fillRect(0, 0, 400, 400);
    c.fillStyle = 'rgba(255,255,255,.85)'; c.font = 'italic 90px Georgia,serif'; c.textAlign = 'center'; c.fillText('♡', 200, 235); return cv;
  }
  var ex = exemple();
  Array.prototype.forEach.call(cartes, function (a) {
    var cv = a.querySelector('canvas'), code = a.getAttribute('data-code');
    if (!cv) return;
    dessiner(cv, code, ex, 'x');
    var im = new Image(); im.onload = function () { dessiner(cv, code, im, '30oz'); }; im.src = a.getAttribute('data-ex');
  });
  Array.prototype.forEach.call(document.querySelectorAll('[data-filtre]'), function (b) {
    b.addEventListener('click', function () {
      var v = b.getAttribute('data-filtre');
      Array.prototype.forEach.call(document.querySelectorAll('[data-filtre]'), function (x) { x.classList.toggle('actif', x === b); });
      Array.prototype.forEach.call(document.querySelectorAll('#grille-prod .prod'), function (a) { a.hidden = v !== 'tous' && a.getAttribute('data-cat') !== v; });
      var ap = document.querySelector('.apercu-vignette'); if (ap) ap.hidden = v !== 'tous';
    });
  });

  Array.prototype.forEach.call(document.querySelectorAll('.coeur-fav'), function (h) { h.addEventListener('click', function () { h.classList.toggle('on'); }); });
  var av = document.querySelector('.apercu-vignette'); if (av) av.addEventListener('click', function () { var t = $('commande-titre'); if (t) t.scrollIntoView({ behavior: 'smooth' }); });
  var selP = $('b-produit'), selV = $('b-variante'), cvA = $('b-apercu');
  function maj() {
    var code = selP.value;
    dessiner(cvA, code || 'tasse', img);
    var p = code && P[code], v = p && p.v[selV.value];
    $('b-r-prod').textContent = v ? p.nom + ' · ' + v[0] + ' : ' + v[1] + ' $' : '—';
    $('b-r-total').innerHTML = '<strong>' + (v ? (v[1] + LIV) + ' $' : '—') + '</strong>';
  }
  function choisir(code) {
    selP.value = code; remplir();
    var t = $('commande-titre'); if (t) t.scrollIntoView({ behavior: 'smooth' });
  }
  function remplir() {
    var p = P[selP.value]; selV.innerHTML = '';
    if (!p) { selV.disabled = true; selV.innerHTML = '<option value="">Choisir d\'abord un produit</option>'; maj(); return; }
    selV.disabled = false;
    Object.keys(p.v).forEach(function (k) { var o = document.createElement('option'); o.value = k; o.textContent = p.v[k][0] + ' · ' + p.v[k][1] + ' $'; selV.appendChild(o); });
    maj();
  }
  selP.addEventListener('change', remplir); selV.addEventListener('change', maj);
  Array.prototype.forEach.call(document.querySelectorAll('[data-choisir]'), function (b) { b.addEventListener('click', function () { choisir(b.getAttribute('data-choisir')); }); });
  Array.prototype.forEach.call(f.querySelectorAll('input[name=cadeau]'), function (r) { r.addEventListener('change', function () { $('b-dest').hidden = f.cadeau.value !== 'oui'; }); });

  $('b-photo').addEventListener('change', function () {
    var fi = this.files[0], al = $('b-alerte'); al.hidden = true; img = null; photoData = null;
    if (!fi) { maj(); return; }
    if (!/^image\/(jpeg|png)$/.test(fi.type)) { al.textContent = 'Choisissez une photo JPEG ou PNG.'; al.hidden = false; this.value = ''; maj(); return; }
    var im = new Image(), url = URL.createObjectURL(fi);
    im.onload = function () {
      var m = Math.min(1, 1600 / Math.max(im.width, im.height)), w = Math.round(im.width * m), h = Math.round(im.height * m);
      var cv = document.createElement('canvas'); cv.width = w; cv.height = h; cv.getContext('2d').drawImage(im, 0, 0, w, h);
      photoData = cv.toDataURL('image/jpeg', .9); img = im; URL.revokeObjectURL(url);
      if (Math.max(im.width, im.height) < 900) { al.textContent = 'Cette photo est de petite taille : l\'impression pourrait manquer de netteté. Une photo plus grande donnerait un meilleur résultat.'; al.hidden = false; }
      maj();
    };
    im.onerror = function () { al.textContent = 'Cette photo n\'a pas pu être lue.'; al.hidden = false; };
    im.src = url;
  });

  function erreur(t) { var e = $('b-erreur'); e.textContent = t; e.hidden = !t; }
  f.addEventListener('submit', function (e) {
    e.preventDefault(); erreur('');
    if (!selP.value || !selV.value) return erreur('Choisissez un produit et un format.');
    if (!photoData) return erreur('Ajoutez votre photo.');
    var d = {}; ['prenom', 'nom', 'courriel', 'telephone', 'adresse'].forEach(function (k) { d[k] = (f[k].value || '').trim(); });
    if (!d.prenom || !d.nom || !d.courriel || !d.telephone || !d.adresse) return erreur('Remplissez tous les champs obligatoires (*).');
    if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(d.courriel)) return erreur('Courriel invalide.');
    d.action = 'commander'; d.produit = selP.value; d.variante = selV.value; d.photo = photoData; d.piege = f.piege.value;
    d.cadeau = f.cadeau.value === 'oui'; d.dest_nom = f.dest_nom.value; d.dest_adresse = f.dest_adresse.value; d.note = f.note.value;
    var b = $('b-envoyer'); b.disabled = true; b.textContent = 'Un instant…';
    fetch((C.serveur || '').replace('norelias-reservation', 'norelias-stripe'), { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(d) })
      .then(function (r) { return r.json(); })
      .then(function (j) { if (j && j.ok && j.paiement) { location.href = j.paiement; } else { throw new Error(j && j.erreur); } })
      .catch(function (er) { erreur((er && er.message) || 'Une erreur est survenue. Écrivez-moi à bonjour@norelias.ca.'); b.disabled = false; b.textContent = 'Payer par carte'; });
  });

  var q = location.search, m = $('msg-boutique');
  if (/merci=1/.test(q)) { m.textContent = 'Merci ! Votre paiement est reçu. Un courriel de confirmation vous attend.'; m.hidden = false; }
  else if (/annule=1/.test(q)) { m.textContent = 'Le paiement a été annulé. Votre commande n\'a pas été complétée.'; m.hidden = false; }
  window.__maj = maj; maj();
})();
