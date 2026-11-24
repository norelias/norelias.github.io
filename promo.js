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

  /* Accueil : grand bloc Black Friday sous la première section */
  var hero = document.querySelector('main .hero');
  if (hero && !document.querySelector('.bf-accueil')) {
    var st2 = document.createElement('style');
    st2.textContent = '.bf-accueil{margin:14px auto;max-width:1040px;width:calc(100% - 24px);box-sizing:border-box;display:flex;align-items:center;justify-content:center;gap:22px;flex-wrap:wrap;text-align:center;background:linear-gradient(180deg,#7a2433,#5c1823);color:#fff;border-radius:18px;padding:18px 26px;box-shadow:0 4px 16px rgba(60,20,25,.25);font-family:Georgia,"Times New Roman",serif}' +
      '.bf-accueil .gros{font-size:56px;line-height:1;font-weight:700;color:#f6d9c8;letter-spacing:-1px}' +
      '.bf-accueil .txt{display:flex;flex-direction:column;gap:3px;align-items:center}.bf-accueil .txt b{font-size:24px;letter-spacing:.5px}.bf-accueil .txt span{font-size:16px;color:#f3e3dc}' +
      '.bf-accueil a{background:#f6d9c8;color:#5c1823;border-radius:99px;padding:10px 22px;font:700 15px Georgia,serif;text-decoration:none;white-space:nowrap}' +
      '@media(max-width:600px){.bf-accueil{gap:4px 12px;padding:8px 12px;border-radius:12px;margin:8px auto}.bf-accueil .gros{font-size:34px}.bf-accueil .txt b{font-size:15px}.bf-accueil .txt span{font-size:11.5px}.bf-accueil a{padding:6px 16px;font-size:13px}}';
    document.head.appendChild(st2);
    var b = document.createElement('section');
    b.className = 'bf-accueil'; b.setAttribute('aria-label', 'Promotion Black Friday');
    b.innerHTML = '<div class="gros">−30 %</div><div class="txt"><b>BLACK FRIDAY</b><span>sur toutes mes séances photo · du 24 au 30 novembre</span></div><a href="' + racine + 'reservation/">Réserver ma séance</a>';
    hero.parentNode.insertBefore(b, hero.nextSibling);
  }
})();
