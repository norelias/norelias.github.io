/* Bandeau Black Friday : −30 % sur toutes les séances, du 24 au 30 novembre 2026 (heure de l'Atlantique). */
(function () {
  var t = Date.now();
  if (t < Date.UTC(2026, 10, 24, 4, 0, 0) || t > Date.UTC(2026, 11, 1, 3, 59, 59)) return;
  if (/\/(stats|confirmer|paiement|merci-personnel)\//.test(location.pathname)) return;
  var racine = (document.currentScript && document.currentScript.src) ? document.currentScript.src.replace(/promo\.js.*$/, '') : '/';
  var st = document.createElement('style');
  st.textContent = '.bf-bandeau{display:flex;justify-content:center;align-items:center;gap:10px;flex-wrap:wrap;background:linear-gradient(180deg,#7a2433,#5c1823);color:#fff;font:600 14px/1.3 Georgia,"Times New Roman",serif;text-align:center;padding:9px 12px;position:relative;z-index:50}' +
    '.bf-bandeau b{color:#f6d9c8}.bf-bandeau a{color:#5c1823;background:#f6d9c8;border-radius:99px;padding:4px 12px;text-decoration:none;font-size:13px;white-space:nowrap}' +
    '@media(max-width:480px){.bf-bandeau{font-size:12.5px;padding:7px 8px;gap:6px}}';
  document.head.appendChild(st);
  var d = document.createElement('div');
  d.className = 'bf-bandeau';
  d.setAttribute('role', 'note');
  d.innerHTML = '<span><b>Black Friday</b> · −30 % sur toutes les séances jusqu\'au 30 novembre</span><a href="' + racine + 'reservation/">Réserver</a>';
  document.body.insertBefore(d, document.body.firstChild);
})();
