// Norélias Photography — outils communs aux fonctions serveur (prix, courriels).
import { createClient } from "npm:@supabase/supabase-js@2";

export const sb = createClient(Deno.env.get("SUPABASE_URL")!, Deno.env.get("SUPABASE_SERVICE_ROLE_KEY")!);
export const SITE = "https://norelias.ca";
export const COURRIEL_SABRINA = "bonjour@norelias.ca";
export const esc = (t: unknown) => String(t ?? "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
export const argent = (n: number | null | undefined) =>
  n == null ? "—" : n.toLocaleString("fr-CA", { minimumFractionDigits: n % 1 ? 2 : 0, maximumFractionDigits: 2 }) + " $";

// Prix (le serveur recalcule toujours : on ne fait pas confiance au navigateur)
const FORFAITS: Record<string, { nom: string; prix: number | null; acompte?: number; parEnfant?: boolean; minimum?: number; parPersonneEnPlus?: number; inclus?: number }> = {
  essentiel: { nom: "Essentiel", prix: 200, acompte: 100 },
  signature: { nom: "Signature", prix: 300, acompte: 150 },
  complete: { nom: "Complète", prix: 450, acompte: 225 },
  noel: { nom: "Mini-séance de Noël", prix: 95, acompte: 95 },
  ecole: { nom: "Garderie / École", prix: 35, parEnfant: true, minimum: 10 },
  finissants: { nom: "Finissants", prix: 175, acompte: 87.5 },
  bal: { nom: "Bal de promos", prix: 150, parPersonneEnPlus: 25, inclus: 2 },
  personnalise: { nom: "Forfait personnalisé", prix: null, acompte: 100 },
};
export function montants(forfait: string, personnes: number) {
  const f = FORFAITS[forfait];
  if (!f) return null;
  if (f.parEnfant) {
    const n = Math.max(f.minimum!, personnes || 0);
    const total = f.prix! * n;
    return { nom: f.nom, total, acompte: Math.round(total * 50) / 100 };
  }
  if (f.parPersonneEnPlus) {
    const total = f.prix! + Math.max(0, (personnes || 1) - f.inclus!) * f.parPersonneEnPlus;
    return { nom: f.nom, total, acompte: Math.round(total * 50) / 100 };
  }
  return { nom: f.nom, total: f.prix, acompte: f.acompte! };
}
export const prenom = (nom: string) => String(nom || "").trim().split(/\s+/)[0] || "";
export function solde(r: any) {
  if (r.forfait === "noel") return "Aucun (payé au complet)";
  if (r.total == null) return "Selon votre proposition";
  return argent(Number(r.total) - Number(r.acompte));
}
export function dateHeure(d: Date) {
  return new Intl.DateTimeFormat("fr-CA", { timeZone: "America/Moncton", weekday: "long", day: "numeric", month: "long", hour: "numeric", minute: "2-digit" }).format(d).replace(":", " h ");
}

// ---------- Gabarit de courriel aux couleurs du site ----------
export function gabarit(o: { petit: string; titre: string; avant: string; lignes?: [string, string][]; titreEncadre?: string; apres?: string; bouton?: [string, string] }) {
  const lignes = (o.lignes ?? []).map(([l, v], i) =>
    `<tr><td style="padding:6px 0;color:#7a645b;width:38%;vertical-align:top;${i ? "border-top:1px solid #f0e2da;" : ""}">${l}</td><td style="padding:6px 0;font-weight:bold;${i ? "border-top:1px solid #f0e2da;" : ""}">${v}</td></tr>`).join("");
  const encadre = lignes ? `<tr><td style="padding:4px 34px 8px;"><table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#fffaf7;border:1px solid #e3cbbf;border-radius:12px;"><tr><td style="padding:18px 22px;"><div style="font-family:Georgia,serif;font-size:20px;color:#8a4f48;text-align:center;margin-bottom:10px;">${o.titreEncadre ?? "Votre rendez-vous"}</div><table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="font-family:Arial,Helvetica,sans-serif;font-size:15px;color:#2b1d18;">${lignes}</table></td></tr></table></td></tr>` : "";
  const bouton = o.bouton ? `<tr><td align="center" style="padding:14px 34px 4px;"><a href="${o.bouton[1]}" style="display:inline-block;background:#cb9483;background-image:linear-gradient(180deg,#e1b2a4,#cb9483);color:#2b1d18;font-family:Arial,Helvetica,sans-serif;font-weight:bold;font-size:15px;text-decoration:none;padding:12px 26px;border-radius:6px;">${o.bouton[0]} &rarr;</a></td></tr>` : "";
  const para = (h: string) => `<tr><td style="padding:14px 34px 4px;font-family:Arial,Helvetica,sans-serif;font-size:15px;line-height:1.6;color:#3a2a24;">${h}</td></tr>`;
  return `<!DOCTYPE html><html lang="fr-CA"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"></head><body style="margin:0;padding:0;background:#ead9cf;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="background:#ead9cf;"><tr><td align="center" style="padding:24px 12px;">
<table role="presentation" width="600" cellpadding="0" cellspacing="0" border="0" style="width:100%;max-width:600px;background:#f7ece6;border-radius:14px;overflow:hidden;font-family:Georgia,'Times New Roman',serif;color:#2b1d18;">
<tr><td align="center" style="background:#f9f1ec;padding:22px 20px 16px;border-bottom:1px solid #ecdcd2;"><img src="${SITE}/images/logo-courriel.png" width="230" alt="Norélias Photography" style="display:block;width:230px;max-width:70%;height:auto;border:0;"></td></tr>
<tr><td style="background:#5c2f2b;padding:22px 30px;text-align:center;"><div style="font-family:Georgia,serif;font-style:italic;font-size:20px;color:#f0c9bd;">${o.petit}</div><div style="font-family:Georgia,serif;font-size:27px;line-height:1.2;color:#ffffff;margin-top:4px;">${o.titre}</div></td></tr>
${para(o.avant)}${encadre}${o.apres ? para(o.apres) : ""}${bouton}
${para('<p style="margin:0;">Au plaisir de créer de beaux souvenirs avec vous,</p>')}
<tr><td style="padding:6px 34px 0;"><img src="${SITE}/images/signature-courriel.png" width="200" alt="Norélias Photography" style="display:block;width:200px;height:auto;border:0;"></td></tr>
<tr><td align="center" style="padding:18px 20px 6px;"><table role="presentation" cellpadding="0" cellspacing="0" border="0"><tr><td style="width:70px;border-top:1px solid #a98374;font-size:0;">&nbsp;</td><td style="padding:0 8px;color:#8a6656;font-size:18px;line-height:1;">&#9825;</td><td style="width:70px;border-top:1px solid #a98374;font-size:0;">&nbsp;</td></tr></table></td></tr>
<tr><td style="background:#3d2622;padding:16px 20px;text-align:center;font-family:Arial,Helvetica,sans-serif;font-size:13px;line-height:1.6;color:#f1dcd4;">Sabrina · Norélias Photography · Campbellton, N.-B.<br><a href="tel:+15067870667" style="color:#f1dcd4;text-decoration:none;">506-787-0667</a> · <a href="mailto:bonjour@norelias.ca" style="color:#f1dcd4;text-decoration:none;">bonjour@norelias.ca</a> · <a href="${SITE}" style="color:#f1dcd4;text-decoration:none;">norelias.ca</a></td></tr>
</table></td></tr></table></body></html>`;
}

export function lignesRdv(r: any, avecAdresse = true): [string, string][] {
  const l: [string, string][] = [["Séance", esc(r.seance)]];
  if (r.ecole) l.push([r.seance_code === "bal" ? "École" : "École / garderie", esc(r.ecole)]);
  if (r.groupes) l.push(["Groupes", esc(r.groupes)]);
  if (r.seance_code === "garderie-ecole" && r.personnes) l.push(["Enfants", esc(r.personnes)]);
  l.push(["Date", esc(r.date_txt)], ["Heure", esc(r.heure)]);
  if (avecAdresse && r.adresse) l.push(["Adresse", esc(r.adresse).replace(/\n/g, "<br>")]);
  return l;
}

// ---------- Envoi (Resend) ----------
async function cleResend() {
  const env = Deno.env.get("RESEND_API_KEY");
  if (env) return env;
  const { data } = await sb.from("secrets_internes").select("valeur").eq("cle", "norelias_resend").maybeSingle();
  return data?.valeur as string | undefined;
}
export async function envoyer(a: string, sujet: string, html: string) {
  const cle = await cleResend();
  if (!cle) { console.log("[courriel non envoyé : clé Resend absente]", a, sujet); return false; }
  const r = await fetch("https://api.resend.com/emails", {
    method: "POST",
    headers: { Authorization: `Bearer ${cle}`, "Content-Type": "application/json" },
    body: JSON.stringify({ from: "Norélias Photography <bonjour@norelias.ca>", to: [a], reply_to: COURRIEL_SABRINA, subject: sujet, html }),
  });
  if (!r.ok) console.log("Resend erreur", r.status, (await r.text()).slice(0, 300));
  return r.ok;
}

// ---------- Les courriels ----------
export const courriels = {
  interac(r: any) {
    const limite = dateHeure(new Date(new Date(r.soumise_le).getTime() + 24 * 3600e3));
    return ["Instructions pour votre virement Interac · Norélias Photography", gabarit({
      petit: `Merci, ${esc(prenom(r.nom))} !`, titre: "Votre réservation est presque terminée",
      avant: `<p style="margin:0 0 12px;">Bonjour ${esc(prenom(r.nom))},</p><p style="margin:0;">Votre plage horaire est réservée pour vous pendant 24 h. Pour la garder, il reste une seule étape : votre virement Interac.</p>`,
      titreEncadre: "Comment faire votre virement",
      lignes: [["Montant", argent(Number(r.acompte))], ["Envoyer à", "bonjour@norelias.ca"], ["Message", `Votre nom et « ${esc(r.seance)} »`], ["Avant le", esc(limite)], ["Séance", esc(r.seance)], ["Date", `${esc(r.date_txt)} à ${esc(r.heure)}`]],
      apres: `<p style="margin:0 0 12px;"><strong>Important :</strong> votre courriel de confirmation vous sera envoyé seulement une fois le virement reçu. Si le virement n'est pas fait dans les 24 h, la réservation sera <strong>annulée automatiquement</strong> et la plage horaire sera libérée.</p><p style="margin:0;">Le solde sera payable le jour de la séance. Une question ? Répondez simplement à ce courriel.</p>`,
    })];
  },
  recue(r: any) {
    return ["Nous avons bien reçu votre réservation · Norélias Photography", gabarit({
      petit: `Merci, ${esc(prenom(r.nom))} !`, titre: "Votre demande est bien reçue",
      avant: `<p style="margin:0 0 12px;">Bonjour ${esc(prenom(r.nom))},</p><p style="margin:0;">Merci pour votre réservation ! Dès que votre paiement est reçu, je vous envoie votre courriel de confirmation avec tous les détails.</p>`,
      lignes: [...lignesRdv(r, false), ["Montant à payer", argent(Number(r.acompte))]],
    })];
  },
  confirmation(r: any) {
    return ["Votre séance est confirmée · Norélias Photography", gabarit({
      petit: `Merci, ${esc(prenom(r.nom))} !`, titre: "Votre séance est confirmée",
      avant: `<p style="margin:0 0 12px;">Bonjour ${esc(prenom(r.nom))},</p><p style="margin:0;">J'ai bien reçu votre paiement et votre date est maintenant réservée juste pour vous. Merci de me faire confiance pour immortaliser ce moment si précieux. J'ai vraiment hâte de vous rencontrer !</p>`,
      lignes: [...lignesRdv(r), ["Paiement reçu", argent(Number(r.acompte))], ["Solde le jour même", solde(r)]],
      apres: `<p style="margin:0 0 12px;">Vous recevrez vos photos en fichiers numériques haute résolution, dans une galerie privée, dans un délai de 3 semaines après la séance.</p><p style="margin:0;">Besoin de changer quelque chose ? Écrivez-moi au moins 48 h à l'avance, selon ma <a href="${SITE}/politique-reservation/" style="color:#8a4f48;">politique de réservation</a>. Je vous enverrai aussi un petit rappel la veille.</p>`,
    })];
  },
  rappel(r: any) {
    return ["C'est demain ! Rappel de votre séance · Norélias Photography", gabarit({
      petit: `Bonjour ${esc(prenom(r.nom))} !`, titre: "Votre séance, c'est demain",
      avant: `<p style="margin:0;">J'ai tellement hâte de vous voir ! Voici un petit rappel de votre rendez-vous.</p>`,
      lignes: [...lignesRdv(r), ["Solde à payer", solde(r)]],
      apres: `<p style="margin:0 0 8px;"><strong>Quelques petits conseils :</strong></p><p style="margin:0 0 4px;">• Choisissez des vêtements aux tons doux et unis.</p><p style="margin:0 0 4px;">• Pour bébé : un petit boire juste avant, c'est parfait.</p><p style="margin:0 0 12px;">• Soyez vous-mêmes, je m'occupe du reste !</p><p style="margin:0;">Un empêchement ? Appelez-moi au 506-787-0667 ou répondez à ce courriel.</p>`,
    })];
  },
  panier(r: any) {
    return ["Votre séance vous attend · Norélias Photography", gabarit({
      petit: `Bonjour ${esc(prenom(r.nom))} !`, titre: "Votre réservation n'est pas terminée",
      avant: `<p style="margin:0;">Vous avez commencé à réserver une séance, mais la réservation n'a pas été complétée. Les places partent vite, surtout en décembre ! Il ne vous reste que quelques clics.</p>`,
      lignes: lignesRdv(r, false), titreEncadre: "Ce que vous aviez choisi",
      bouton: ["Terminer ma réservation", `${SITE}/reservation/${r.seance_code ? "?seance=" + encodeURIComponent(r.seance_code) : ""}`],
      apres: `<p style="margin:0;">Une question avant de réserver ? Répondez simplement à ce courriel, ça me fera plaisir de vous répondre.</p>`,
    })];
  },
  annulation(r: any) {
    return ["Réservation annulée · Norélias Photography", gabarit({
      petit: `Bonjour ${esc(prenom(r.nom))},`, titre: "Votre réservation a été annulée",
      avant: `<p style="margin:0;">Je n'ai pas reçu votre virement Interac dans les 24 h suivant votre réservation. Comme prévu, la réservation a donc été annulée automatiquement et la plage horaire a été libérée.</p>`,
      lignes: lignesRdv(r, false), titreEncadre: "Réservation annulée",
      bouton: ["Réserver à nouveau", `${SITE}/reservation/`],
      apres: `<p style="margin:0;">S'il s'agit d'une erreur ou si vous avez fait le virement, répondez à ce courriel et je regarde ça avec vous tout de suite.</p>`,
    })];
  },
  sabrina(r: any, quoi: string) {
    const lien = `${SITE}/confirmer/?id=${r.id}&j=${r.jeton}`;
    return [`${quoi} : ${r.seance} · ${r.nom} · ${r.date_txt}`, gabarit({
      petit: "Bonjour Sabrina !", titre: esc(quoi),
      avant: `<p style="margin:0;">Voici les détails :</p>`, titreEncadre: "Réservation",
      lignes: [...lignesRdv(r, false), ["Nom", esc(r.nom)], ["Courriel", esc(r.courriel)], ["Téléphone", esc(r.telephone)], ["Personnes", esc(r.personnes)], ["Paiement", r.mode === "interac" ? "Virement Interac" : "Carte (Stripe)"], ["Acompte", argent(Number(r.acompte))], ["Total", r.total == null ? "Sur demande" : argent(Number(r.total))], ["Portfolio", esc(r.portfolio)], ["Message", esc(r.message).replace(/\n/g, "<br>")]],
      apres: r.statut === "en_attente" ? `<p style="margin:0;">Quand le paiement est reçu, cliquez sur le bouton : vous entrez l'adresse de la séance et la cliente reçoit sa confirmation.${r.mode === "interac" ? " Sans virement dans les 24 h, la réservation s'annulera toute seule." : ""}</p>` : "",
      bouton: r.statut === "en_attente" ? ["Paiement reçu : confirmer", lien] : undefined,
    })];
  },
};
