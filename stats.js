/* Norélias Photography — statistiques de visites (sans cookie, sans adresse IP conservée). */
(function () {
  var URL_STATS = "https://evoivwqcxohfwaxlumvj.supabase.co/functions/v1/norelias-stats";
  function lire(k, s) { try { return (s ? sessionStorage : localStorage).getItem(k); } catch (e) { return null; } }
  function ecrire(k, v, s) { try { (s ? sessionStorage : localStorage).setItem(k, v); } catch (e) {} }
  var q = location.search;
  if (/[?&]stats=off/.test(q)) ecrire("norelias_moi", "1");
  if (/[?&]stats=on/.test(q)) { try { localStorage.removeItem("norelias_moi"); } catch (e) {} }
  if (lire("norelias_moi") === "1") return;
  if (navigator.webdriver || /bot|crawl|spider|slurp|headless|preview|lighthouse|pingdom|monitor/i.test(navigator.userAgent || "")) return;
  if (/^(localhost|127\.)/.test(location.hostname) || location.protocol === "file:") return;
  function id() { var a = new Uint8Array(12); (window.crypto || {}).getRandomValues ? crypto.getRandomValues(a) : a.forEach(function (_, i) { a[i] = Math.random() * 256; }); return Array.prototype.map.call(a, function (x) { return ("0" + x.toString(16)).slice(-2); }).join(""); }
  var vid = lire("norelias_vid") || id(); ecrire("norelias_vid", vid);
  var sid = lire("norelias_sid", true) || id(); ecrire("norelias_sid", sid, true);
  var vueId = null, secondes = 0, dernier = document.hidden ? 0 : Date.now();
  function envoyer(o, garder) {
    try { return fetch(URL_STATS, { method: "POST", headers: { "Content-Type": "text/plain" }, body: JSON.stringify(o), keepalive: !!garder }).then(function (r) { return r.json(); }); } catch (e) { return Promise.resolve({}); }
  }
  function cumul() { if (dernier) { secondes += Math.round((Date.now() - dernier) / 1000); dernier = Date.now(); } return secondes; }
  function battre(garder) { if (vueId) envoyer({ action: "battement", id: vueId, secondes: cumul() }, garder); }
  envoyer({ action: "vue", sid: sid, vid: vid, page: location.pathname, titre: document.title, ref: document.referrer }).then(function (r) { if (r && r.id) vueId = r.id; });
  setInterval(function () { if (!document.hidden) battre(false); }, 15000);
  document.addEventListener("visibilitychange", function () {
    if (document.hidden) { cumul(); dernier = 0; battre(true); } else { dernier = Date.now(); }
  });
  window.addEventListener("pagehide", function () { cumul(); battre(true); });
})();
