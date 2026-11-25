/* Black Friday : −30 % sur toutes les séances, du 24 au 30 novembre 2026 (heure de l'Atlantique).
   Annonce discrète : un petit rond sur l'accueil et sur la boutique. */
(function () {
  var t = Date.now();
  if (t < Date.UTC(2026, 10, 24, 4, 0, 0) || t > Date.UTC(2026, 11, 1, 3, 59, 59)) return;
  if (!/^\/(boutique\/?)?(index\.html)?$/.test(location.pathname)) return;
  var racine = (document.currentScript && document.currentScript.src) ? document.currentScript.src.replace(/promo\.js.*$/, '') : '/';
  var st = document.createElement('style');
  st.textContent = '.bf-rond{position:fixed;right:14px;top:78px;z-index:60;width:78px;height:78px;border-radius:50%;display:flex;flex-direction:column;align-items:center;justify-content:center;text-decoration:none;color:#f6e7dc;background:linear-gradient(160deg,#9a2434,#6a1624);border:2px solid #ecc478;box-shadow:0 4px 14px rgba(60,15,22,.4);font-family:Georgia,"Times New Roman",serif;line-height:1;text-align:center;transition:transform .2s}' +
    '.bf-rond:hover{transform:scale(1.06)}.bf-rond b{font-size:22px;font-weight:700;letter-spacing:-.5px}.bf-rond span{font-size:9.5px;letter-spacing:.6px;text-transform:uppercase;margin-top:3px;color:#ecc478}' +
    '@media(max-width:600px){.bf-rond{width:64px;height:64px;right:8px;top:56px}.bf-rond b{font-size:18px}.bf-rond span{font-size:8px}}';
  document.head.appendChild(st);
  var a = document.createElement('a');
  a.className = 'bf-rond'; a.href = racine + 'reservation/';
  a.setAttribute('aria-label', 'Black Friday : moins 30 % sur les séances photo, du 24 au 30 novembre');
  a.title = 'Black Friday · −30 % sur les séances photo jusqu\'au 30 novembre';
  a.innerHTML = '<b>−30 %</b><span>Black Friday</span>';
  document.body.appendChild(a);
})();
