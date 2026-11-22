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
    personnalise: { nom: 'Forfait personnalisé', prix: null, acompte: null }
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
    var dateChamp = form.elements.date;
    dateChamp.setAttribute('aria-label', form.elements.seance.value === 'Nouveau-né' ? "Date prévue de l'accouchement" : 'Date souhaitée');
  }
  form.elements.seance.addEventListener('change', synchroniserNoel);
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
    var perso = form.elements.forfait.value === 'personnalise';
    document.getElementById('recap-seance').textContent = form.elements.seance.value;
    document.getElementById('recap-forfait').textContent = f.nom;
    document.getElementById('recap-date').textContent = form.elements.date.value + ' · ' + form.elements.heure.value;
    document.getElementById('recap-total').textContent = perso ? 'Sur demande' : argent(f.prix);
    var acompte = document.getElementById('recap-acompte');
    acompte.textContent = perso ? 'Aucun pour le moment' : (form.elements.forfait.value === 'noel' ? argent(f.acompte) + ' (paiement complet)' : argent(f.acompte) + ' (50 %)');
    document.getElementById('zone-paiement').hidden = perso;
    document.getElementById('note-perso').hidden = !perso;
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
    var perso = cle === 'personnalise';
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
      portfolio: form.elements.portfolio.checked ? 'Oui' : 'Non',
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
      document.getElementById('fin-perso').hidden = mode !== 'aucun';
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

  // Date minimale : demain
  var demain = new Date(Date.now() + 864e5);
  form.elements.date.min = demain.toISOString().slice(0, 10);
  document.querySelectorAll('.courriel-interac').forEach(function (el) { el.textContent = C.interac; });
})();
