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

  function synchroniserNoel() {
    var estNoel = form.elements.seance.value === 'Mini-séance de Noël';
    if (estNoel) form.elements.forfait.value = 'noel';
    Array.prototype.forEach.call(form.elements.forfait.options, function (o) {
      o.disabled = estNoel ? o.value !== 'noel' : o.value === 'noel';
    });
    if (!estNoel && form.elements.forfait.value === 'noel') form.elements.forfait.value = '';
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
  function choisirDate(d) {
    choisie = d;
    form.elements.date.value = JOURS[d.getDay()] + ' ' + d.getDate() + ' ' + MOIS[d.getMonth()] + ' ' + d.getFullYear();
    form.elements.dateIso.value = iso(d);
    form.elements.heure.value = '';
    dessiner();
    var zone = cal.querySelector('.creneaux'), liste = cal.querySelector('.creneaux-liste');
    zone.hidden = false;
    cal.querySelector('.creneaux-titre').textContent = 'Heures disponibles le ' + form.elements.date.value;
    liste.innerHTML = '';
    heuresDu(d).forEach(function (h) {
      var b = document.createElement('button');
      b.type = 'button'; b.textContent = h; b.setAttribute('aria-pressed', 'false');
      b.addEventListener('click', function () {
        form.elements.heure.value = h;
        liste.querySelectorAll('button').forEach(function (x) { x.setAttribute('aria-pressed', x === b ? 'true' : 'false'); });
        document.querySelector('.erreur-cal').hidden = true;
      });
      liste.appendChild(b);
    });
  }
  window.majCalendrier = function () {
    if (!cal) return;
    if (choisie && !heuresDu(choisie).length) { choisie = null; form.elements.date.value = ''; form.elements.heure.value = ''; cal.querySelector('.creneaux').hidden = true; }
    var p = premiereDispo();
    if (!choisie && p) vue = new Date(p.getFullYear(), p.getMonth(), 1);
    if (choisie) choisirDate(choisie); else dessiner();
  };
  if (cal) cal.querySelectorAll('.cal-nav').forEach(function (b) {
    b.addEventListener('click', function () { vue = new Date(vue.getFullYear(), vue.getMonth() + Number(b.dataset.mois), 1); dessiner(); });
  });
  synchroniserNoel();

  function argent(n) { return n.toLocaleString('fr-CA') + ' $'; }

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
    if (i === 0 && form.elements.seance.value && (!form.elements.date.value || !form.elements.heure.value)) {
      document.querySelector('.erreur-cal').hidden = false; return false;
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
      // Forfait personnalisé : pas de paiement en ligne, on passe à l'envoi de la demande
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
    document.getElementById('recap-total').textContent = estPerso ? 'Sur demande' : argent(f.prix);
    var acompte = document.getElementById('recap-acompte');
    acompte.textContent = estPerso ? argent(f.acompte) + ' (dépôt déduit du total)' : (form.elements.forfait.value === 'noel' ? argent(f.acompte) + ' (paiement complet)' : argent(f.acompte) + ' (50 %)');
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
    if (f && f.acompte) {
      document.querySelectorAll('.montant-interac').forEach(function (el) { el.textContent = argent(f.acompte); });
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
    if (mode === 'carte' && !lienStripe) mode = 'interac'; // lien Stripe pas encore configuré

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
      montant: f.acompte ? argent(f.acompte) : 'Sur demande'
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
    envoyer(C.emailjs && C.emailjs.modeleReservation, d, 'Réservation — ' + d.seance + ' — ' + d.nom, corps).then(function () {
      document.getElementById('fin-carte').hidden = mode !== 'carte';
      document.getElementById('fin-interac').hidden = mode !== 'interac';
      document.getElementById('fin-perso').hidden = cle !== 'personnalise';
      afficher(3);
      if (mode === 'carte') {
        var url = lienStripe + (lienStripe.indexOf('?') > -1 ? '&' : '?') + 'prefilled_email=' + encodeURIComponent(d.courriel);
        document.getElementById('lien-stripe').href = url;
        setTimeout(function () { window.location.href = url; }, 1500);
      }
    }).catch(function () {
      alert("Oups, la demande n'a pas pu partir. Écrivez-moi à " + C.courriel + ' ou appelez-moi au ' + C.telephone + '.');
      bouton.disabled = false;
    });
  });

  document.querySelectorAll('.courriel-interac').forEach(function (el) { el.textContent = C.interac; });
})();
