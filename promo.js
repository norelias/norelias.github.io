/* Black Friday : −30 % sur toutes les séances, du 24 au 30 novembre 2026 (heure de l'Atlantique).
   Accueil : bandeau en haut, ovale « −30 % » sur la photo, grand bandeau au-dessus de « Magie de Noël ».
   Autres pages : bandeau en haut. Disparaît tout seul après le 30 novembre. */
(function () {
  var t = Date.now();
  if (t < Date.UTC(2026, 10, 24, 4, 0, 0) || t > Date.UTC(2026, 11, 1, 3, 59, 59)) return;
  if (/\/(stats|confirmer|paiement|merci-personnel)\//.test(location.pathname)) return;
  var racine = (document.currentScript && document.currentScript.src) ? document.currentScript.src.replace(/promo\.js.*$/, '') : '/';
  var DATES = 'Du 24 au 30 novembre';
  var ROUGE = 'linear-gradient(100deg,#842834,#641620 65%,#651921)';
  var CADEAU = '<svg viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" width="20" height="20"><rect x="4" y="11" width="24" height="18" rx="2"/><path d="M3 16h26M16 11v18M16 11c-7-1-9-4-7-6 3-4 7 1 7 6zm0 0c7-1 9-4 7-6-3-4-7 1-7 6z"/></svg>';
  var st = document.createElement('style');
  st.textContent =
    '.bf-haut{display:flex;justify-content:center;align-items:center;gap:12px;flex-wrap:wrap;background:' + ROUGE + ';color:#fff7f0;font:600 15px/1.3 Georgia,"Times New Roman",serif;text-align:center;padding:9px 14px;position:relative;z-index:50;letter-spacing:.2px}' +
    '.bf-haut b{letter-spacing:1px;font-size:16px}.bf-haut i{font-style:normal;opacity:.95}.bf-haut .pt{opacity:.7}' +
    '.bf-haut a{background:#f7ede4;color:#641620;border-radius:99px;padding:5px 16px;text-decoration:none;font-size:14px;font-weight:700;white-space:nowrap}' +
    '.bf-ovale{position:absolute;left:59%;top:52%;transform:translate(-50%,-50%) rotate(-4deg);z-index:3;width:132px;height:74px;border-radius:50%;display:flex;align-items:center;justify-content:center;text-decoration:none;color:#fff3ea;background:' + ROUGE + ';border:2px solid #f3d6cc;box-shadow:0 0 0 3px #7c2530,0 4px 12px rgba(60,15,22,.35);font:400 46px/1 Allura,cursive}' +
    '.bf-band .christmas-title p{font:700 26px/1.1 Georgia,"Times New Roman",serif;margin:8px 0 0}.bf-band .christmas-title p small{font:italic 400 17px/1.3 Georgia,"Times New Roman",serif;margin-top:6px;opacity:1}' +
    '.bf-band .christmas-right{align-items:center;padding-left:0}.bf-band .christmas-right .btn{width:236px;height:48px;font-size:17px;gap:10px;display:inline-flex;align-items:center;justify-content:center}' +
    '@media(max-width:760px){.bf-haut{font-size:11px;gap:4px 8px;padding:6px 8px}.bf-haut b{font-size:11.5px}.bf-haut a{font-size:10.5px;padding:3px 10px}.bf-haut .pt{display:none}' +
    '.bf-ovale{left:74%;top:56%;width:78px;height:44px;font-size:28px;border-width:1.5px;box-shadow:0 0 0 2px #7c2530,0 3px 8px rgba(60,15,22,.35)}' +
    '.bf-band .christmas-title p{font-size:11px;margin:3px 0 0}.bf-band .christmas-title p small{font-size:9px;margin-top:2px}' +
    '.bf-band .christmas-right{align-items:center;padding:0 6px;margin:7px 0}.bf-band .christmas-right .btn{width:98px;height:30px;font-size:8.5px;gap:3px;padding:0 4px}.bf-band .christmas-right .btn svg{width:11px;height:11px}}';
  document.head.appendChild(st);

  var haut = document.createElement('div');
  haut.className = 'bf-haut'; haut.setAttribute('role', 'note');
  haut.innerHTML = '<b>BLACK FRIDAY</b><span class="pt">•</span><span>−30 % sur les séances photo</span><span class="pt">•</span><i>' + DATES + '</i><a href="' + racine + 'reservation/">Profiter du rabais →</a>';

  var main = document.querySelector('main'), hero = main && main.querySelector('.hero'), noel = main && main.querySelector('.christmas');
  if (hero) {
    main.insertBefore(haut, main.firstChild);
    var o = document.createElement('a');
    o.className = 'bf-ovale'; o.href = racine + 'reservation/'; o.textContent = '−30 %';
    o.setAttribute('aria-label', 'Black Friday : moins 30 % sur les séances photo, du 24 au 30 novembre');
    hero.appendChild(o);
    if (noel) {
      var b = noel.cloneNode(true);
      b.className = 'christmas bf-band'; b.removeAttribute('aria-labelledby');
      var img = b.querySelector('.christmas-photo img'); if (img) img.alt = 'Petite fille en robe de velours rouge devant le sapin de Noël';
      var ti = b.querySelector('.christmas-title');
      ti.innerHTML = '<h2>Black Friday <img class="decor-white-heart" src="' + racine + 'images/d-coeur-noel.png" alt=""></h2><p>−30 % sur les séances photo<small class="dates-noel">' + DATES + '</small></p>';
      var dr = b.querySelector('.christmas-right');
      dr.innerHTML = '<a class="btn btn-cream" href="' + racine + 'reservation/">' + CADEAU + 'Profiter du rabais <span class="next">→</span></a>';
      var fl = b.querySelector('.snowflake'); if (fl) fl.remove();
      noel.parentNode.insertBefore(b, noel);
    }
  } else {
    document.body.insertBefore(haut, document.body.firstChild);
  }
})();
