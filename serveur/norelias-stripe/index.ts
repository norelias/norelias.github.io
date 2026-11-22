// Norélias Photography — reçoit les avis de Stripe (paiement réussi ou page de paiement expirée).
// Sécurité : on ne croit jamais le contenu reçu ; on redemande la session directement à Stripe avec la clé secrète.
import { sb, envoyer, courriels, stripe, COURRIEL_SABRINA } from "../commun.ts";

Deno.serve(async (req) => {
  if (req.method !== "POST") return new Response("ok");
  let ev: any = {};
  try { ev = await req.json(); } catch (_) { return new Response("invalide", { status: 400 }); }
  const id = ev?.data?.object?.id;
  if (!id || !String(id).startsWith("cs_")) return new Response("ignoré");
  const session = await stripe("checkout/sessions/" + encodeURIComponent(id));
  if (!session) return new Response("session introuvable", { status: 400 });
  const resId = session.client_reference_id;
  const { data: r } = await sb.from("norelias_reservations").select("*").eq("id", resId).maybeSingle();
  if (!r) return new Response("réservation introuvable");

  if (session.payment_status === "paid" && !r.payee_le) {
    const { data: r2 } = await sb.from("norelias_reservations").update({ payee_le: new Date().toISOString(), stripe_session: id }).eq("id", r.id).select("*").single();
    const [s1, h1] = courriels.payee(r2);
    const [s2, h2] = courriels.sabrina(r2, "Nouvelle réservation payée par carte");
    await Promise.all([envoyer(r2.courriel, s1, h1), envoyer(COURRIEL_SABRINA, s2, h2)]);
  } else if (session.status === "expired" && r.statut === "en_attente" && !r.payee_le && r.stripe_session === id) {
    const { data: r2 } = await sb.from("norelias_reservations").update({ statut: "annulee", annulee_le: new Date().toISOString() }).eq("id", r.id).eq("statut", "en_attente").select("*").maybeSingle();
    if (r2) {
      const [s1, h1] = courriels.annulation(r2);
      const [s2, h2] = courriels.sabrina(r2, "Réservation annulée (paiement par carte jamais complété)");
      await Promise.all([envoyer(r2.courriel, s1, h1), envoyer(COURRIEL_SABRINA, s2, h2)]);
    }
  }
  return new Response("ok");
});
