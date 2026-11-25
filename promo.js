/* Black Friday : −30 % sur toutes les séances, du 24 au 30 novembre 2026 (heure de l'Atlantique).
   Emblème (image) sur la photo de l'accueil. Disparaît tout seul après le 30 novembre. */
(function () {
  var t = Date.now();
  if (t < Date.UTC(2026, 10, 24, 4, 0, 0) || t > Date.UTC(2026, 11, 1, 3, 59, 59)) return;
  var hero = document.querySelector('main .hero');
  if (!hero) return;
  var racine = (document.currentScript && document.currentScript.src) ? document.currentScript.src.replace(/promo\.js.*$/, '') : '/';
  var st = document.createElement('style');
  st.textContent =
    '.bf-emb{position:absolute;left:66%;top:-1px;transform:translateX(-50%);z-index:3;display:block;height:232px;filter:drop-shadow(0 4px 10px rgba(80,30,20,.22))}' +
    '.bf-emb img{display:block;height:100%;width:auto}' +
    '@media(max-width:760px){.bf-emb{left:66%;top:2px;height:124px}}';
  document.head.appendChild(st);
  var a = document.createElement('a');
  a.className = 'bf-emb'; a.href = racine + 'reservation/';
  a.setAttribute('aria-label', 'Black Friday : moins 30 % sur toutes les séances photo, du 24 au 30 novembre');
  a.innerHTML = '<img src="' + racine + 'images/black-friday-rond.webp?v=1" alt="Black Friday, moins 30 % sur toutes les séances photo, du 24 au 30 novembre">';
  hero.appendChild(a);
})();
