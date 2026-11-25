/* Black Friday : −30 % sur toutes les séances, du 24 au 30 novembre 2026 (heure de l'Atlantique).
   Emblème rond sur la photo de l'accueil. Disparaît tout seul après le 30 novembre. */
(function () {
  var t = Date.now();
  if (t < Date.UTC(2026, 10, 24, 4, 0, 0) || t > Date.UTC(2026, 11, 1, 3, 59, 59)) return;
  var hero = document.querySelector('main .hero');
  if (!hero) return;
  var racine = (document.currentScript && document.currentScript.src) ? document.currentScript.src.replace(/promo\.js.*$/, '') : '/';
  var BR = '#8c5a3c', ROUGE = '#7c1f2b';
  var branche = function (sens) {
    var f = function (x, y, r) { return '<ellipse cx="' + x + '" cy="' + y + '" rx="3.2" ry="8" transform="rotate(' + r + ' ' + x + ' ' + y + ')"/>'; };
    return '<svg class="bf-lau ' + sens + '" viewBox="0 0 40 150" aria-hidden="true" fill="' + BR + '"><path d="M30 146C10 112 6 60 22 6" fill="none" stroke="' + BR + '" stroke-width="1.6"/>' +
      f(14, 118, -40) + f(24, 104, 35) + f(10, 94, -45) + f(22, 80, 35) + f(9, 68, -45) + f(21, 54, 35) + f(11, 42, -40) + f(23, 30, 35) + f(17, 16, -15) + '</svg>';
  };
  var st = document.createElement('style');
  st.textContent =
    '.bf-emb{position:absolute;left:59%;top:2px;transform:translateX(-50%);z-index:3;width:190px;text-decoration:none;color:' + ROUGE + ';text-align:center;font-family:Georgia,"Times New Roman",serif}' +
    '.bf-rond{position:relative;width:184px;height:184px;border-radius:50%;background:radial-gradient(circle at 50% 45%,#fffaf4 0,#fffaf4 68%,rgba(255,250,244,.9) 82%,rgba(255,250,244,.55) 100%);box-shadow:0 4px 18px rgba(90,40,30,.28);display:flex;flex-direction:column;align-items:center;justify-content:center;padding:0 24px;box-sizing:border-box}' +
    '.bf-lau{position:absolute;top:18px;height:132px;width:36px;opacity:.9}.bf-lau.g{left:8px}.bf-lau.d{right:8px;transform:scaleX(-1)}' +
    '.bf-emb .bf-t1{font:400 40px/.9 Allura,cursive;margin:0}.bf-t1 span{font-size:.6em;vertical-align:middle}' +
    '.bf-emb .bf-t2{font:700 44px/1 Georgia,"Times New Roman",serif;letter-spacing:-1px;margin:2px 0 0}' +
    '.bf-emb .bf-t3{font:600 13.5px/1.15 Georgia,"Times New Roman",serif;color:#5a2f27;margin:3px 0 0}' +
    '.bf-emb .bf-date{display:inline-flex;align-items:center;gap:7px;margin-top:-16px;position:relative;background:linear-gradient(100deg,#842834,#641620);color:#fff3ea;border-radius:99px;padding:7px 18px;font:700 14px/1 Georgia,"Times New Roman",serif;white-space:nowrap;box-shadow:0 3px 8px rgba(60,15,22,.35)}' +
    '.bf-date svg{width:15px;height:15px}' +
    '@media(max-width:760px){.bf-emb{left:64%;width:104px;top:3px}.bf-rond{width:96px;height:96px;padding:0 11px}.bf-lau{top:9px;height:72px;width:20px}.bf-lau.g{left:4px}.bf-lau.d{right:4px}' +
    '.bf-emb .bf-t1{font-size:23px}.bf-t2{font-size:29px}.bf-t3{font-size:8px}.bf-emb .bf-date{margin-top:-9px;padding:4px 8px;font-size:8px;gap:4px}.bf-date svg{width:9px;height:9px}}';
  document.head.appendChild(st);
  var a = document.createElement('a');
  a.className = 'bf-emb'; a.href = racine + 'reservation/';
  a.setAttribute('aria-label', 'Black Friday : moins 30 % sur toutes les séances photo, du 24 au 30 novembre');
  a.innerHTML = '<div class="bf-rond">' + branche('g') + branche('d') + '<p class="bf-t1">Black Friday <span>♡</span></p><p class="bf-t2">−30 %</p><p class="bf-t3">sur toutes les<br>séances photo</p></div>' +
    '<span class="bf-date"><svg viewBox="0 0 32 32" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><rect x="4" y="7" width="24" height="21" rx="3"/><path d="M4 14h24M10 4v6M22 4v6"/></svg>Du 24 au 30 novembre</span>';
  hero.appendChild(a);
})();
