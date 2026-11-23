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
  if (m0 && /merci=1/.test(q0)) { m0.textContent = 'Merci ! Votre paiement est reçu. Un courriel de confirmation vous attend.'; m0.hidden = false; }
  else if (m0 && /annule=1/.test(q0)) { m0.textContent = "Le paiement a été annulé. Votre commande n'a pas été complétée."; m0.hidden = false; }

  var f = $('form-produit');
  if (!f) return;
  var C = window.NORELIAS || {};
  var P = JSON.parse(f.getAttribute('data-produit'));
  var CODE = f.getAttribute('data-code'), LIV = Number(f.getAttribute('data-livraison'));
  var img = null, photoData = null, vue = 0;
  var cvA = $('pp-apercu'), qte = $('pp-qte'), fi = $('pp-photo');
  function radios() { return f.querySelectorAll('input[name=variante]'); }
  function variante() { var r = f.querySelector('input[name=variante]:checked'); return r ? r.value : Object.keys(P.v)[0]; }
  function cover(ctx, im, x, y, w, h) {
    var r = Math.max(w / im.width, h / im.height), sw = w / r, sh = h / r;
    ctx.drawImage(im, (im.width - sw) / 2, (im.height - sh) / 2, sw, sh, x, y, w, h);
  }
  var racine = (document.querySelector('link[href*="style.css"]').getAttribute('href') || '').replace(/style\.css.*$/, '');

  /* ---- Aperçu réaliste : la photo de la cliente est posée sur une vraie photo du produit ---- */
  var BASES = {
    tasse: { src: 'images/pp-tasse.jpg', zone: { x: 263, y: 101, w: 267, h: 296, r: 6 }, vues: [{}, { miroir: 1 }, { zoom: [396, 250, 1.75] }] },
    isotherme: { src: 'images/pp-isotherme.jpg', zone: { x: 333, y: 76, w: 234, h: 394, r: 4, fondu: [350, 440] }, vues: [{}, { miroir: 1 }, { zoom: [448, 240, 1.75] }] },
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
      if (face) { c.save(); heart(c, cx, hy + 1, S * .9); c.clip(); if (im) cover(c, im, cx - S, hy - S * .66, S * 2, S * 1.84); else { c.fillStyle = '#d9c7bb'; c.fillRect(cx - S, hy - S, S * 2, S * 2.2); } c.restore(); }
      else { heart(c, cx, hy + 1, S * .9); var g = c.createLinearGradient(cx - S, hy - S * .6, cx + S, hy + S); g.addColorStop(0, '#cfd0d2'); g.addColorStop(.5, '#e9eaeb'); g.addColorStop(1, '#b4b6b8'); c.fillStyle = g; c.fill(); }
    } else if (v === 'rect') {
      anneau(c, cx, cy - 188, 36); rr(c, cx - 22, cy - 160, 44, 36, 8); c.fillStyle = argent(c, cx - 22, cy - 160, cx + 22, cy - 124); c.fill();
      rr(c, cx - 100, cy - 128, 200, 262, 12); c.fillStyle = argent(c, cx - 100, cy - 128, cx + 100, cy + 134); c.fill(); c.shadowColor = 'transparent';
      if (face) { c.save(); rr(c, cx - 86, cy - 114, 172, 234, 6); c.clip(); if (im) cover(c, im, cx - 86, cy - 114, 172, 234); else { c.fillStyle = '#d9c7bb'; c.fillRect(cx - 90, cy - 118, 180, 242); } c.restore(); }
      else { rr(c, cx - 86, cy - 114, 172, 234, 6); c.fillStyle = '#dcdddf'; c.fill(); }
    } else {
      anneau(c, cx, cy - 196, 36);
      rr(c, cx - 36, cy - 168, 72, 80, 14); c.fillStyle = '#1b191a'; c.fill();
      rr(c, cx - 104, cy - 112, 208, 262, 22); var g2 = c.createLinearGradient(cx - 104, cy - 112, cx + 104, cy + 150); g2.addColorStop(0, '#343032'); g2.addColorStop(.5, '#1b191a'); g2.addColorStop(1, '#2a2628'); c.fillStyle = g2; c.fill(); c.shadowColor = 'transparent';
      c.beginPath(); c.arc(cx, cy - 128, 12, 0, 7); c.fillStyle = argent(c, cx - 12, cy - 140, cx + 12, cy - 116); c.fill();
      c.setLineDash([5, 4]); c.lineWidth = 1.6; c.strokeStyle = '#8a8284'; rr(c, cx - 94, cy - 102, 188, 242, 16); c.stroke(); c.setLineDash([]);
      if (face) { c.save(); rr(c, cx - 70, cy - 66, 140, 180, 5); c.clip(); if (im) cover(c, im, cx - 70, cy - 66, 140, 180); else { c.fillStyle = '#d9c7bb'; c.fillRect(cx - 72, cy - 68, 144, 184); } c.restore(); c.lineWidth = 3; c.strokeStyle = '#4a4446'; rr(c, cx - 70, cy - 66, 140, 180, 5); c.stroke(); }
    }
    c.restore();
  }
  function pastille(c, x, y, t) {
    c.save(); rr(c, x - 52, y - 17, 104, 34, 17); c.fillStyle = 'rgba(250,243,239,.96)'; c.shadowColor = 'rgba(70,45,35,.2)'; c.shadowBlur = 8; c.fill(); c.shadowColor = 'transparent';
    c.fillStyle = '#3a2a24'; c.font = '600 19px Nunito, Arial, sans-serif'; c.textAlign = 'center'; c.textBaseline = 'middle'; c.fillText(t, x, y + 1); c.restore();
  }

  function dessiner(cv, cle, im, o) {
    o = o || {};
    var b = BASES[cle]; if (!b || !b.pret) return;
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
      if (z.fondu) {
        cover(c, im, zx - 1, z.y - 1, z.w + 2, z.fondu[1] - z.y + 40);
        var g0 = c.createLinearGradient(0, z.fondu[0], 0, z.fondu[1] + 18); g0.addColorStop(0, 'rgba(243,240,241,0)'); g0.addColorStop(1, 'rgba(243,240,241,1)');
        c.fillStyle = g0; c.fillRect(zx, z.fondu[0], z.w, z.h); c.fillStyle = '#f3f0f1'; c.fillRect(zx, z.fondu[1] + 17, z.w, z.h);
      } else cover(c, im, zx - 1, z.y - 1, z.w + 2, z.h + 2);
      var g = c.createLinearGradient(zx, 0, zx + z.w, 0), s = function (p) { return m > 0 ? p : 1 - p; };
      g.addColorStop(0, 'rgba(40,25,20,.34)'); g.addColorStop(.16, 'rgba(40,25,20,0)'); g.addColorStop(s(.62), 'rgba(255,255,255,.07)'); g.addColorStop(.86, 'rgba(40,25,20,0)'); g.addColorStop(1, 'rgba(40,25,20,.4)');
      c.fillStyle = g; c.fillRect(zx, z.y, z.w, z.h);
      var vg = c.createLinearGradient(0, z.y, 0, z.y + z.h); vg.addColorStop(0, 'rgba(40,25,20,.12)'); vg.addColorStop(.1, 'rgba(40,25,20,0)'); vg.addColorStop(.92, 'rgba(40,25,20,0)'); vg.addColorStop(1, 'rgba(40,25,20,.16)');
      c.fillStyle = vg; c.fillRect(zx, z.y, z.w, z.h); c.restore();
    } else if (b.rond) {
      var q = b.rond; c.save(); c.beginPath(); c.arc(q.x, q.y, q.r, 0, 7); c.clip(); cover(c, im, q.x - q.r, q.y - q.r, q.r * 2, q.r * 2);
      var rg = c.createRadialGradient(q.x - q.r * .35, q.y - q.r * .45, q.r * .05, q.x, q.y, q.r);
      rg.addColorStop(0, 'rgba(255,255,255,.3)'); rg.addColorStop(.35, 'rgba(255,255,255,.04)'); rg.addColorStop(.85, 'rgba(255,255,255,0)'); rg.addColorStop(1, 'rgba(255,255,255,.22)');
      c.fillStyle = rg; c.fillRect(q.x - q.r, q.y - q.r, q.r * 2, q.r * 2); c.restore();
    }
    c.restore();
  }

  /* ---- galerie : vue principale + vignettes dessinées avec la photo de la cliente ---- */
  var vitrine = $('pp-vues');
  function echelle() { var i = Array.prototype.indexOf.call(radios(), f.querySelector('input[name=variante]:checked')); return CODE === 'tasse' ? 1 + .08 * Math.max(0, i) : CODE === 'isotherme' ? 1 + .09 * Math.max(0, i) : 1; }
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

  /* ---- prix, taxes, adresses ---- */
  var TX = { QC: { tps: 5, tvq: 9.975 }, NB: { tvh: 15 }, NL: { tvh: 15 }, NS: { tvh: 14 }, PE: { tvh: 15 }, ON: { tvh: 13 }, AB: { tps: 5 }, BC: { tps: 5 }, MB: { tps: 5 }, SK: { tps: 5 }, NT: { tps: 5 }, NU: { tps: 5 }, YT: { tps: 5 } };
  var PREM = { A: 'NL', B: 'NS', C: 'PE', E: 'NB', G: 'QC', H: 'QC', J: 'QC', K: 'ON', L: 'ON', M: 'ON', N: 'ON', P: 'ON', R: 'MB', S: 'SK', T: 'AB', V: 'BC', X: 'NT', Y: 'YT' };
  var NOMS = { 'quebec': 'QC', 'new brunswick': 'NB', 'nouveau-brunswick': 'NB', 'nova scotia': 'NS', 'nouvelle-ecosse': 'NS', 'ontario': 'ON', 'manitoba': 'MB', 'saskatchewan': 'SK', 'alberta': 'AB', 'british columbia': 'BC', 'colombie-britannique': 'BC', 'prince edward island': 'PE', 'ile-du-prince-edouard': 'PE', 'newfoundland and labrador': 'NL', 'terre-neuve-et-labrador': 'NL', 'newfoundland': 'NL', 'yukon': 'YT', 'northwest territories': 'NT', 'territoires du nord-ouest': 'NT', 'nunavut': 'NU' };
  function provDe(nom) { var s = String(nom || '').normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase().trim(); return NOMS[s] || (TX[s.toUpperCase()] ? s.toUpperCase() : ''); }
  function cpOk(prov, cp) { var c = String(cp || '').toUpperCase().replace(/\s/g, ''); if (!/^[A-Z]\d[A-Z]\d[A-Z]\d$/.test(c)) return false; var p = PREM[c[0]]; return p === prov || (p === 'NT' && prov === 'NU'); }
  function fmt(n) { return (Math.round(n * 100) / 100).toLocaleString('fr-CA', { minimumFractionDigits: 2 }) + ' $'; }
  function provTaxes() { return (f.cadeau.value === 'oui' && $('d-adresse').value.trim()) ? $('d-prov').value : $('b-prov').value; }
  function maj() {
    dessinerVues();
    var v = P.v[variante()] || P.v[Object.keys(P.v)[0]], q = Math.max(1, Math.min(10, Number(qte.value) || 1)), sous = v[1] * q, base = sous + LIV;
    $('pp-prix').textContent = fmt(v[1]);
    $('pp-total-prod').textContent = fmt(sous);
    var badge = $('pp-badge-txt'); if (badge && CODE === 'tasse') badge.innerHTML = '<b>' + v[0].split(' (')[0] + '</b><small>(' + (v[0].split(' (')[1] || '').replace(')', '') + ')</small>';
    Array.prototype.forEach.call(document.querySelectorAll('[data-modele]'), function (bt) { bt.classList.toggle('on', bt.getAttribute('data-modele') === variante()); });
    var prov = provTaxes(), t = TX[prov], lig = '<dt>Sous-total (' + q + ' article' + (q > 1 ? 's' : '') + ')</dt><dd>' + fmt(sous) + '</dd><dt>Livraison par la poste</dt><dd>' + fmt(LIV) + '</dd>', tot = base;
    function ajout(nom, p) { var m = Math.round(base * p) / 100; tot += m; lig += '<dt>' + nom + '</dt><dd>' + fmt(m) + '</dd>'; }
    if (t) { if (t.tps) ajout('TPS (5 %)', t.tps); if (t.tvq) ajout('TVQ (9,975 %)', t.tvq); if (t.tvh) ajout('TVH (' + t.tvh + ' %)', t.tvh); }
    else lig += '<dt class="gris">Taxes</dt><dd class="gris">selon votre province</dd>';
    tot = Math.round(tot * 100) / 100;
    $('pp-totaux').innerHTML = lig + '<dt class="t">Total</dt><dd class="t">' + fmt(tot) + (t ? '' : ' + taxes') + '</dd>';
    $('pp-btn-total').textContent = fmt(tot) + (t ? '' : ' + taxes');
  }
  window.__maj = maj;
  Array.prototype.forEach.call(radios(), function (r) { r.addEventListener('change', maj); });
  Array.prototype.forEach.call(document.querySelectorAll('[data-qte]'), function (b) { b.addEventListener('click', function () { qte.value = Math.max(1, Math.min(10, (Number(qte.value) || 1) + Number(b.getAttribute('data-qte')))); maj(); }); });
  qte.addEventListener('input', maj);
  $('pp-texte').addEventListener('input', function () { $('pp-compte').textContent = this.value.length + ' / 50'; });
  Array.prototype.forEach.call(f.querySelectorAll('input[name=cadeau]'), function (r) { r.addEventListener('change', function () { $('b-dest').hidden = f.cadeau.value !== 'oui'; maj(); }); });
  var liste = $('pp-liste'); if (liste) liste.addEventListener('click', function () { liste.classList.toggle('on'); });

  // adresses : suggestions partout au Canada (Photon / OpenStreetMap)
  function adresse(p) {
    var rue = $(p + '-adresse'), ville = $(p + '-ville'), prov = $(p + '-prov'), cp = $(p + '-cp'), ul = $(p + '-liste'), items = [], cur = -1, t, ctl;
    function fermer() { ul.hidden = true; ul.innerHTML = ''; items = []; cur = -1; rue.setAttribute('aria-expanded', 'false'); }
    function choisir(i) { var a = items[i]; if (!a) return; rue.value = a.rue; ville.value = a.ville; if (a.prov) prov.value = a.prov; cp.value = a.cp; fermer(); maj(); (a.cp ? cp : ville).focus(); }
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
    prov.addEventListener('change', maj);
    cp.addEventListener('input', function () {
      var v = cp.value.toUpperCase().replace(/[^A-Z0-9 ]/g, ''); if (v.replace(' ', '').length > 3 && v.indexOf(' ') < 0) v = v.slice(0, 3) + ' ' + v.slice(3); cp.value = v.slice(0, 7);
      if (!prov.value && PREM[v[0]]) { prov.value = PREM[v[0]]; maj(); }
    });
  }
  adresse('b'); adresse('d');

  /* ---- téléversement ---- */
  var zone = $('pp-zone'), vign = $('pp-vignette'), alerte = $('pp-alerte');
  function effacer() { img = null; photoData = null; fi.value = ''; vign.hidden = true; zone.hidden = false; alerte.hidden = true; maj(); }
  $('pp-changer').addEventListener('click', function () { fi.click(); });
  ['dragover', 'dragenter'].forEach(function (ev) { zone.addEventListener(ev, function (e) { e.preventDefault(); zone.classList.add('survol'); }); });
  ['dragleave', 'drop'].forEach(function (ev) { zone.addEventListener(ev, function (e) { e.preventDefault(); zone.classList.remove('survol'); }); });
  zone.addEventListener('drop', function (e) { if (e.dataTransfer.files[0]) { fi.files = e.dataTransfer.files; lire(); } });
  fi.addEventListener('change', lire);
  function lire() {
    var fic = fi.files[0]; alerte.hidden = true;
    if (!fic) return;
    if (!/^image\/(jpeg|png)$/.test(fic.type)) { alerte.textContent = 'Choisissez une photo JPEG ou PNG.'; alerte.hidden = false; fi.value = ''; return; }
    if (fic.size > 10 * 1024 * 1024) { alerte.textContent = 'La photo est trop lourde (maximum 10 Mo).'; alerte.hidden = false; fi.value = ''; return; }
    var im = new Image(), url = URL.createObjectURL(fic);
    im.onload = function () {
      var m = Math.min(1, 1600 / Math.max(im.width, im.height)), w = Math.round(im.width * m), h = Math.round(im.height * m);
      var cv = document.createElement('canvas'); cv.width = w; cv.height = h; cv.getContext('2d').drawImage(im, 0, 0, w, h);
      photoData = cv.toDataURL('image/jpeg', .9); img = im; $('pp-miniature').src = photoData; vign.hidden = false; zone.hidden = true;
      $('pp-nom').textContent = fic.name; $('pp-taille').textContent = fic.size > 1048576 ? (fic.size / 1048576).toFixed(1).replace('.', ',') + ' Mo' : Math.round(fic.size / 1024) + ' Ko';
      if (Math.max(im.width, im.height) < 900) { alerte.textContent = "Cette photo est de petite taille : l'impression pourrait manquer de netteté. Une photo plus grande donnerait un meilleur résultat."; alerte.hidden = false; }
      maj();
    };
    im.onerror = function () { alerte.textContent = "Cette photo n'a pas pu être lue."; alerte.hidden = false; };
    im.src = url;
  }

  /* ---- commander ---- */
  function erreur(t) { var e = $('b-erreur'); e.textContent = t; e.hidden = !t; if (t) e.scrollIntoView({ block: 'center', behavior: 'smooth' }); }
  $('pp-commander').addEventListener('click', function () {
    if (!photoData) { alerte.textContent = "Ajoutez d'abord votre photo."; alerte.hidden = false; zone.hidden ? vign.scrollIntoView({ block: 'center' }) : zone.scrollIntoView({ block: 'center', behavior: 'smooth' }); return; }
    $('commande').scrollIntoView({ behavior: 'smooth', block: 'start' });
  });
  f.addEventListener('submit', function (e) {
    e.preventDefault(); erreur('');
    if (!photoData) return erreur("Ajoutez votre photo (haut de la page).");
    var d = {}; ['prenom', 'nom', 'courriel', 'telephone', 'adresse', 'ville', 'province', 'code_postal'].forEach(function (k) { d[k] = (f[k].value || '').trim(); });
    if (!d.prenom || !d.nom || !d.courriel || !d.telephone || !d.adresse || !d.ville || !d.province) return erreur('Remplissez tous les champs obligatoires (*).');
    if (!/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(d.courriel)) return erreur('Courriel invalide.');
    if (!cpOk(d.province, d.code_postal)) return erreur('Vérifiez votre code postal : il doit correspondre à la province (ex. : E3N 2S1).');
    d.action = 'commander'; d.produit = CODE; d.variante = variante(); d.quantite = Number(qte.value) || 1; d.texte = $('pp-texte').value; d.photo = photoData; d.piege = f.piege.value;
    d.cadeau = f.cadeau.value === 'oui'; d.dest_nom = f.dest_nom.value; d.note = f.note.value;
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
  maj();
})();
