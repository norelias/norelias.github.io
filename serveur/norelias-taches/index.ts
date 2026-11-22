// Norélias Photography — tâches automatiques (appelées toutes les 15 minutes par pg_cron)
// 1) rappel « réservation pas terminée » (3 h après) · 2) annulation des virements Interac non reçus en 24 h · 3) rappel 24 h avant la séance
import { sb, envoyer, courriels, COURRIEL_SABRINA } from "../commun.ts";

Deno.serve(async (req) => {
  const { data: s } = await sb.from("secrets_internes").select("valeur").eq("cle", "norelias_taches").maybeSingle();
  if (!s?.valeur || req.headers.get("x-secret") !== s.valeur) return new Response("non autorisé", { status: 401 });
  const maintenant = Date.now(), iso = (ms: number) => new Date(ms).toISOString();
  const bilan = { panier: 0, annulees: 0, rappels: 0 };

  // 1) Réservations commencées mais pas terminées : un seul rappel, entre 3 h et 48 h après
  const { data: brouillons } = await sb.from("norelias_reservations").select("*").eq("statut", "brouillon").is("rappel_panier_le", null)
    .not("courriel", "is", null).lt("maj_le", iso(maintenant - 3 * 3600e3)).gt("cree_le", iso(maintenant - 48 * 3600e3));
  for (const r of brouillons ?? []) {
    // pas de rappel si la même personne a terminé une autre réservation depuis
    const { data: autre } = await sb.from("norelias_reservations").select("id").eq("courriel", r.courriel).neq("statut", "brouillon").gte("cree_le", r.cree_le).limit(1);
    await sb.from("norelias_reservations").update({ rappel_panier_le: iso(maintenant) }).eq("id", r.id);
    if (autre?.length) continue;
    const [sj, h] = courriels.panier(r);
    if (await envoyer(r.courriel, sj, h)) bilan.panier++;
  }

  // 2) Virements Interac non reçus après 24 h : annulation automatique
  const { data: expirees } = await sb.from("norelias_reservations").select("*").eq("statut", "en_attente").eq("mode", "interac")
    .lt("soumise_le", iso(maintenant - 24 * 3600e3));
  for (const r of expirees ?? []) {
    const { data: r2 } = await sb.from("norelias_reservations").update({ statut: "annulee", annulee_le: iso(maintenant) }).eq("id", r.id).eq("statut", "en_attente").select("*").maybeSingle();
    if (!r2) continue;
    const [sj, h] = courriels.annulation(r2);
    const [sj2, h2] = courriels.sabrina(r2, "Réservation annulée automatiquement (virement non reçu)");
    await Promise.all([envoyer(r2.courriel, sj, h), envoyer(COURRIEL_SABRINA, sj2, h2)]);
    bilan.annulees++;
  }

  // 3) Rappel la veille : séances confirmées qui commencent dans les prochaines 24 h
  const { data: demain } = await sb.from("norelias_reservations").select("*").eq("statut", "confirmee").is("rappel_24h_le", null)
    .gt("debut", iso(maintenant + 2 * 3600e3)).lte("debut", iso(maintenant + 24 * 3600e3));
  for (const r of demain ?? []) {
    await sb.from("norelias_reservations").update({ rappel_24h_le: iso(maintenant) }).eq("id", r.id);
    const [sj, h] = courriels.rappel(r);
    if (await envoyer(r.courriel, sj, h)) bilan.rappels++;
  }
  return new Response(JSON.stringify(bilan), { headers: { "content-type": "application/json" } });
});
