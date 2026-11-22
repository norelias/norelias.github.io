// Norélias Photography — réservations du site norelias.ca
// Actions : brouillon (réservation commencée), soumettre, creneaux (plages prises), lire / confirmer / annuler (Sabrina, avec jeton).
import { sb, envoyer, courriels, montants, COURRIEL_SABRINA } from "../commun.ts";

const ORIGINES = ["https://norelias.ca", "https://www.norelias.ca", "http://norelias.ca", "https://norelias.github.io", "http://localhost:8765"];
const cors = (o: string | null) => ({
  "Access-Control-Allow-Origin": o && ORIGINES.includes(o) ? o : "https://norelias.ca",
  "Access-Control-Allow-Headers": "content-type, apikey, authorization, x-client-info",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Vary": "Origin",
});
const txt = (v: unknown, max = 200) => (v == null ? null : String(v).trim().slice(0, max) || null);
const UUID = /^[0-9a-f-]{36}$/i;

Deno.serve(async (req) => {
  const H = cors(req.headers.get("Origin"));
  const rep = (o: unknown, s = 200) => new Response(JSON.stringify(o), { status: s, headers: { ...H, "Content-Type": "application/json" } });
  if (req.method === "OPTIONS") return new Response("ok", { headers: H });
  try {
    if (req.method === "GET") {
      // plages déjà prises (en attente de paiement ou confirmées), à partir d'aujourd'hui
      const { data } = await sb.from("norelias_reservations").select("date_iso, heure")
        .in("statut", ["en_attente", "confirmee"]).gte("date_iso", new Date(Date.now() - 864e5).toISOString().slice(0, 10));
      const prises: Record<string, string[]> = {};
      for (const r of data ?? []) (prises[r.date_iso] ??= []).push(r.heure);
      return rep({ prises });
    }
    const b = await req.json();
    if (b.piege) return rep({ ok: true }); // robot

    const champs = () => {
      const personnes = Math.max(0, Math.min(500, parseInt(b.personnes, 10) || 0)) || null;
      return {
        seance: txt(b.seance, 60), seance_code: txt(b.seance_code, 40), forfait: txt(b.forfait, 30),
        date_iso: /^\d{4}-\d{2}-\d{2}$/.test(b.date_iso ?? "") ? b.date_iso : null, date_txt: txt(b.date_txt, 60), heure: txt(b.heure, 20),
        nom: txt(b.nom, 120), courriel: txt(b.courriel, 160)?.toLowerCase() ?? null, telephone: txt(b.telephone, 40),
        personnes, message: txt(b.message, 3000), portfolio: txt(b.portfolio, 10),
        ecole: txt(b.ecole, 150), groupes: Math.max(0, Math.min(100, parseInt(b.groupes, 10) || 0)) || null,
      };
    };

    if (b.action === "brouillon") {
      const c = champs();
      if (!c.courriel || !/^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(c.courriel)) return rep({ ok: false });
      if (b.id && UUID.test(b.id)) {
        const { data } = await sb.from("norelias_reservations").update(c).eq("id", b.id).eq("statut", "brouillon").select("id").maybeSingle();
        if (data) return rep({ ok: true, id: data.id });
      }
      const { data, error } = await sb.from("norelias_reservations").insert({ ...c, statut: "brouillon" }).select("id").single();
      if (error) throw error;
      return rep({ ok: true, id: data.id });
    }

    if (b.action === "soumettre") {
      const c = champs();
      if (!c.nom || !c.courriel || !c.date_iso || !c.heure || !c.seance || !c.forfait) return rep({ ok: false, erreur: "Champs manquants." }, 400);
      const m = montants(c.forfait, c.personnes ?? 0);
      if (!m) return rep({ ok: false, erreur: "Forfait inconnu." }, 400);
      const mode = b.mode === "carte" ? "carte" : "interac";
      // plage déjà prise ?
      const { data: pris } = await sb.from("norelias_reservations").select("id").in("statut", ["en_attente", "confirmee"])
        .eq("date_iso", c.date_iso).eq("heure", c.heure).neq("id", UUID.test(b.id ?? "") ? b.id : "00000000-0000-0000-0000-000000000000").limit(1);
      if (pris?.length) return rep({ ok: false, erreur: "Désolée, cette plage horaire vient d'être réservée. Choisissez une autre heure." }, 409);
      const valeurs = { ...c, mode, total: m.total, acompte: m.acompte, statut: "en_attente", soumise_le: new Date().toISOString() };
      let r: any = null;
      if (b.id && UUID.test(b.id)) {
        const { data } = await sb.from("norelias_reservations").update(valeurs).eq("id", b.id).eq("statut", "brouillon").select("*").maybeSingle();
        r = data;
      }
      if (!r) {
        const { data, error } = await sb.from("norelias_reservations").insert(valeurs).select("*").single();
        if (error) throw error;
        r = data;
      }
      const [s1, h1] = mode === "interac" ? courriels.interac(r) : courriels.recue(r);
      const [s2, h2] = courriels.sabrina(r, mode === "interac" ? "Nouvelle réservation (virement Interac à recevoir)" : "Nouvelle réservation (paiement par carte)");
      await Promise.all([envoyer(r.courriel, s1, h1), envoyer(COURRIEL_SABRINA, s2, h2)]);
      return rep({ ok: true, id: r.id, total: m.total, acompte: m.acompte });
    }

    // ----- Actions de Sabrina (lien secret reçu par courriel) -----
    if (!UUID.test(b.id ?? "") || !UUID.test(b.j ?? "")) return rep({ ok: false, erreur: "Lien invalide." }, 403);
    const { data: r } = await sb.from("norelias_reservations").select("*").eq("id", b.id).eq("jeton", b.j).maybeSingle();
    if (!r) return rep({ ok: false, erreur: "Réservation introuvable." }, 404);

    if (b.action === "lire") return rep({ ok: true, r });

    if (b.action === "confirmer") {
      const adresse = txt(b.adresse, 300);
      if (!adresse) return rep({ ok: false, erreur: "Entrez l'adresse de la séance." }, 400);
      const { data: r2 } = await sb.from("norelias_reservations").update({ statut: "confirmee", adresse, confirmee_le: new Date().toISOString(), annulee_le: null })
        .eq("id", r.id).select("*").single();
      const [s, h] = courriels.confirmation(r2);
      const ok = await envoyer(r2.courriel, s, h);
      return rep({ ok: true, envoye: ok, r: r2 });
    }

    if (b.action === "annuler") {
      const { data: r2 } = await sb.from("norelias_reservations").update({ statut: "annulee", annulee_le: new Date().toISOString() }).eq("id", r.id).select("*").single();
      return rep({ ok: true, r: r2 });
    }
    return rep({ ok: false, erreur: "Action inconnue." }, 400);
  } catch (e) {
    console.log("erreur", e);
    return rep({ ok: false, erreur: "Erreur du serveur." }, 500);
  }
});
