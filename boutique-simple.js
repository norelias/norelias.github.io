/* Boutique : produits sans téléversement (calendriers, packs). Utilise le panier de script.js (window.NBQ). */
(function () {
  var f = document.getElementById('form-simple'); if (!f || !window.NBQ) return;
  var $ = function (id) { return document.getElementById(id); }, P = JSON.parse(f.getAttribute('data-produit')), CODE = f.getAttribute('data-code');
  var qte = $('pp-qte'), err = $('pp-erreur');
  function fmt(n) { return (Math.round(n * 100) / 100).toLocaleString('fr-CA', { minimumFractionDigits: 2 }) + ' $'; }
  function choix() { return f.querySelector('input[name=variante]:checked'); }
  function maj() {
    var r = choix(), q = Math.max(1, Math.min(10, Number(qte.value) || 1)), p = Number(r.getAttribute('data-prix'));
    $('pp-prix').textContent = fmt(p); $('pp-total-prod').textContent = fmt(p * q); $('ps-image').src = r.getAttribute('data-img');
    Array.prototype.forEach.call(f.querySelectorAll('.pp-var'), function (l) { l.classList.toggle('on', l.contains(r)); });
  }
  f.addEventListener('change', maj);
  Array.prototype.forEach.call(f.querySelectorAll('[data-qte]'), function (b) { b.addEventListener('click', function () { qte.value = Math.max(1, Math.min(10, (Number(qte.value) || 1) + Number(b.getAttribute('data-qte')))); maj(); }); });
  qte.addEventListener('input', maj);
  $('pp-ajouter').addEventListener('click', function () {
    var pr = $('ps-precis'), r = choix(), v = pr.value.trim();
    if (pr.required && !v) { err.textContent = "Indiquez le nom de l'enfant et la pose choisie."; err.hidden = false; pr.focus(); return; }
    err.hidden = true;
    var item = { id: 'i' + Date.now().toString(36) + Math.random().toString(36).slice(2, 6), code: CODE, variante: r.value, quantite: Math.max(1, Math.min(10, Number(qte.value) || 1)),
      texte: '', police: '', couleur: '', position: '', cadrage: v, precisions: v, sans: 1, apercu: r.getAttribute('data-img').replace('../../', '/').replace('.jpg', '-mini.jpg') };
    var a = window.NBQ.lire(); a.push(item);
    if (!window.NBQ.ecrire(a)) { err.textContent = "Le panier est plein : finalisez ou retirez un article, puis réessayez."; err.hidden = false; return; }
    var ok = $('pp-ajoute'); ok.hidden = false; ok.scrollIntoView({ block: 'center', behavior: 'smooth' });
  });
  maj();
})();
