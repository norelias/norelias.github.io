//! Modèles de courriels transactionnels : bienvenue, vérification, récupération, sécurité, facturation, suppression
//! du compte, notification de changement d'adresse. FR/EN, responsives (colonne de 600 px qui s'adapte), aux couleurs de
//! Norélias Studio Pro, version texte intégrée. **Aucune image distante, aucun pixel de suivi** ; toutes les variables sont échappées.
//! Aucun courriel marketing ici : le consentement marketing est distinct et ces modèles n'en dépendent pas.

use crate::mail::{MailKind, Message};
use norelias_core::Lang;
use norelias_i18n::{format::{format_date_long, format_money}, Locale};

#[derive(Clone, Debug)]
pub struct Brand { pub product: String, /// Adresse du site, en HTTPS.
    pub base_url: String, pub support_email: String, /// Adresse postale de l'expéditeur (mention légale).
    pub legal_address: String, pub region: String }
impl Default for Brand {
    fn default() -> Self { Self { product: "Norélias Studio Pro".into(), base_url: "https://norelias.ca".into(), support_email: "soutien@norelias.ca".into(), legal_address: "Norélias Photography".into(), region: "CA".into() } }
}

#[derive(Clone, Debug)]
pub enum SecurityEvent { NewDevice { device: String, when: String }, PasswordChanged, TwoFactorEnabled, TwoFactorDisabled, EmailChangeRequested { new_email: String }, EmailChanged { new_email: String }, AccountExists }

#[derive(Clone, Debug)]
pub enum Template {
    Verification { name: Option<String>, code: String, minutes: u32 },
    Welcome { name: Option<String> },
    PasswordRecovery { code: String, minutes: u32 },
    Security(SecurityEvent),
    Billing { invoice_no: String, lines: Vec<(String, i64)>, taxes: Vec<(String, i64)>, total: i64, currency: String, date: (i32, u32, u32) },
    AccountDeletion { effective: (i32, u32, u32) },
    EmailChangeNotice { new_email_masked: String },
}

struct Content { subject: String, preheader: String, title: String, paras: Vec<String>, code: Option<String>, button: Option<(String, String)>, table: Vec<(String, String)>, note: String }

pub fn esc(s: &str) -> String { s.chars().map(|c| match c { '&' => "&amp;".into(), '<' => "&lt;".into(), '>' => "&gt;".into(), '"' => "&quot;".into(), '\'' => "&#39;".into(), c => c.to_string() }).collect() }

/// « jean.tremblay@exemple.ca » → « j***@exemple.ca ».
pub fn mask_email(e: &str) -> String {
    match e.split_once('@') { Some((l, d)) if !l.is_empty() => format!("{}***@{d}", l.chars().next().unwrap()), _ => "***".into() }
}

fn hello(name: &Option<String>, fr: bool) -> String {
    match name.as_deref().map(str::trim).filter(|n| !n.is_empty()) { Some(n) => format!("{} {n},", if fr { "Bonjour" } else { "Hello" }), None => (if fr { "Bonjour," } else { "Hello," }).into() }
}

fn content(t: &Template, lang: Lang, b: &Brand) -> Content {
    let fr = lang == Lang::Fr;
    let loc = Locale::new(if fr { "fr" } else { "en" }, Some(&b.region));
    let date = |d: (i32, u32, u32)| format_date_long(d.0, d.1, d.2, &loc).unwrap_or_default();
    let tx = |a: &str, e: &str| (if fr { a } else { e }).to_string();
    let ignore = tx("Si vous n'êtes pas à l'origine de cette demande, ignorez ce courriel : votre compte reste protégé.", "If you did not make this request, ignore this email: your account stays protected.");
    let never = tx("Norélias ne vous demandera jamais ce code par téléphone ou par message.", "Norélias will never ask you for this code by phone or message.");
    match t {
        Template::Verification { name, code, minutes } => Content {
            subject: tx(&format!("{code} est votre code de vérification"), &format!("{code} is your verification code")),
            preheader: tx(&format!("Votre code expire dans {minutes} minutes."), &format!("Your code expires in {minutes} minutes.")),
            title: tx("Vérifiez votre adresse courriel", "Verify your email address"),
            paras: vec![hello(name, fr), tx(&format!("Entrez ce code dans {} pour activer votre compte. Il expire dans {minutes} minutes et ne fonctionne qu'une fois.", b.product), &format!("Enter this code in {} to activate your account. It expires in {minutes} minutes and works only once.", b.product))],
            code: Some(code.clone()), button: None, table: vec![], note: format!("{ignore} {never}") },
        Template::Welcome { name } => Content {
            subject: tx(&format!("Bienvenue dans {}", b.product), &format!("Welcome to {}", b.product)), preheader: tx("Votre compte est activé.", "Your account is active."),
            title: tx("Bienvenue !", "Welcome!"),
            paras: vec![hello(name, fr), tx("Votre adresse est vérifiée et votre compte est activé. Vos photographies et vos réglages restent sur votre ordinateur ; rien n'est envoyé sans votre consentement explicite.", "Your address is verified and your account is active. Your photographs and settings stay on your computer; nothing is sent without your explicit consent.")],
            code: None, button: Some((tx("Ouvrir l'Académie", "Open the Academy"), format!("{}/academie", b.base_url))), table: vec![], note: tx(&format!("Besoin d'aide ? Écrivez à {}.", b.support_email), &format!("Need help? Write to {}.", b.support_email)) },
        Template::PasswordRecovery { code, minutes } => Content {
            subject: tx(&format!("{code} est votre code de récupération"), &format!("{code} is your recovery code")), preheader: tx(&format!("Expire dans {minutes} minutes."), &format!("Expires in {minutes} minutes.")),
            title: tx("Réinitialiser votre mot de passe", "Reset your password"),
            paras: vec![tx(&format!("Une réinitialisation du mot de passe a été demandée pour votre compte. Entrez ce code dans {} ; il expire dans {minutes} minutes.", b.product), &format!("A password reset was requested for your account. Enter this code in {}; it expires in {minutes} minutes.", b.product))],
            code: Some(code.clone()), button: None, table: vec![], note: format!("{ignore} {never}") },
        Template::Security(ev) => {
            let (subject, title, para, table): (String, String, String, Vec<(String, String)>) = match ev {
                SecurityEvent::NewDevice { device, when } => (tx("Nouvelle connexion à votre compte", "New sign-in to your account"), tx("Nouvelle connexion", "New sign-in"), tx("Une connexion à votre compte vient d'avoir lieu depuis un nouvel appareil.", "Your account was just signed in to from a new device."), vec![(tx("Appareil", "Device"), device.clone()), (tx("Date", "Date"), when.clone())]),
                SecurityEvent::PasswordChanged => (tx("Votre mot de passe a été modifié", "Your password was changed"), tx("Mot de passe modifié", "Password changed"), tx("Le mot de passe de votre compte vient d'être modifié. Toutes vos sessions ont été fermées.", "Your account password was just changed. All your sessions were signed out."), vec![]),
                SecurityEvent::TwoFactorEnabled => (tx("Authentification à deux facteurs activée", "Two-factor authentication enabled"), tx("2FA activée", "2FA enabled"), tx("L'authentification à deux facteurs est maintenant active sur votre compte.", "Two-factor authentication is now active on your account."), vec![]),
                SecurityEvent::TwoFactorDisabled => (tx("Authentification à deux facteurs désactivée", "Two-factor authentication disabled"), tx("2FA désactivée", "2FA disabled"), tx("L'authentification à deux facteurs a été désactivée sur votre compte.", "Two-factor authentication was disabled on your account."), vec![]),
                SecurityEvent::EmailChangeRequested { new_email } => (tx("Demande de changement d'adresse courriel", "Email address change requested"), tx("Changement d'adresse demandé", "Address change requested"), tx("Un changement de l'adresse de votre compte a été demandé. Rien ne change tant que la nouvelle adresse n'est pas vérifiée.", "A change of your account email address was requested. Nothing changes until the new address is verified."), vec![(tx("Nouvelle adresse", "New address"), mask_email(new_email))]),
                SecurityEvent::EmailChanged { new_email } => (tx("L'adresse de votre compte a changé", "Your account email address changed"), tx("Adresse modifiée", "Address changed"), tx("L'adresse courriel de votre compte a été remplacée après vérification.", "Your account email address was replaced after verification."), vec![(tx("Nouvelle adresse", "New address"), mask_email(new_email))]),
                SecurityEvent::AccountExists => (tx("Une inscription a été tentée avec votre adresse", "A sign-up was attempted with your address"), tx("Vous avez déjà un compte", "You already have an account"), tx("Quelqu'un a tenté de créer un compte avec cette adresse, qui est déjà utilisée. Si c'était vous, connectez-vous ou réinitialisez votre mot de passe.", "Someone tried to create an account with this address, which is already in use. If it was you, sign in or reset your password."), vec![]),
            };
            Content { subject, preheader: tx("Alerte de sécurité.", "Security alert."), title, paras: vec![para], code: None, button: None, table,
                note: tx(&format!("Ce n'était pas vous ? Changez votre mot de passe sans attendre et écrivez à {}.", b.support_email), &format!("Not you? Change your password right away and write to {}.", b.support_email)) }
        }
        Template::Billing { invoice_no, lines, taxes, total, currency, date: d } => {
            let mut table: Vec<(String, String)> = lines.iter().map(|(l, c)| (l.clone(), format_money(*c, currency, &loc))).collect();
            table.extend(taxes.iter().map(|(l, c)| (l.clone(), format_money(*c, currency, &loc))));
            table.push((tx("Total", "Total"), format_money(*total, currency, &loc)));
            Content { subject: tx(&format!("Votre reçu {invoice_no}"), &format!("Your receipt {invoice_no}")), preheader: format_money(*total, currency, &loc), title: tx("Reçu de paiement", "Payment receipt"),
                paras: vec![tx(&format!("Merci ! Reçu {invoice_no} du {}.", date(*d)), &format!("Thank you! Receipt {invoice_no} dated {}.", date(*d)))], code: None, button: None, table,
                note: tx(&format!("Une question sur cette facture ? Écrivez à {}.", b.support_email), &format!("A question about this invoice? Write to {}.", b.support_email)) }
        }
        Template::AccountDeletion { effective } => Content {
            subject: tx("Suppression de votre compte", "Your account deletion"), preheader: tx("Confirmation de suppression.", "Deletion confirmation."), title: tx("Votre compte sera supprimé", "Your account will be deleted"),
            paras: vec![tx(&format!("Nous avons reçu votre demande. Votre compte et les données associées seront supprimés le {}, sauf les informations que la loi nous oblige à conserver (par exemple les factures).", date(*effective)),
                           &format!("We received your request. Your account and associated data will be deleted on {}, except information we are legally required to keep (for example invoices).", date(*effective)))],
            code: None, button: None, table: vec![], note: tx(&format!("Vous avez changé d'avis ? Écrivez à {} avant cette date.", b.support_email), &format!("Changed your mind? Write to {} before that date.", b.support_email)) },
        Template::EmailChangeNotice { new_email_masked } => Content {
            subject: tx("L'adresse de votre compte va changer", "Your account email address is changing"), preheader: tx("Alerte de sécurité.", "Security alert."), title: tx("Changement d'adresse en cours", "Address change in progress"),
            paras: vec![tx("Une demande de remplacement de cette adresse par une autre a été faite depuis votre compte.", "A request to replace this address with another one was made from your account.")],
            code: None, button: None, table: vec![(tx("Nouvelle adresse", "New address"), new_email_masked.clone())], note: tx(&format!("Ce n'était pas vous ? Changez votre mot de passe et écrivez à {}.", b.support_email), &format!("Not you? Change your password and write to {}.", b.support_email)) },
    }
}

fn kind_of(t: &Template) -> MailKind {
    match t { Template::Verification { .. } => MailKind::Verification, Template::Welcome { .. } => MailKind::Welcome, Template::PasswordRecovery { .. } => MailKind::PasswordRecovery,
        Template::Security(_) => MailKind::Security, Template::Billing { .. } => MailKind::Billing, Template::AccountDeletion { .. } => MailKind::AccountDeletion, Template::EmailChangeNotice { .. } => MailKind::EmailChangeNotice }
}

fn html(c: &Content, lang: Lang, b: &Brand) -> String {
    let (lang_attr, footer_note) = if lang == Lang::Fr { ("fr", "Ce courriel est transactionnel : il concerne votre compte et ne dépend pas de vos préférences marketing.") } else { ("en", "This is a transactional email about your account; it does not depend on your marketing preferences.") };
    let paras: String = c.paras.iter().map(|p| format!("<p style=\"margin:0 0 16px;font-size:16px;line-height:1.55;color:#F1F1F1\">{}</p>", esc(p))).collect();
    let code = c.code.as_ref().map(|k| format!("<div style=\"margin:8px 0 20px;padding:18px;background:#2C2D30;border-radius:12px;text-align:center;font-family:'SFMono-Regular',Consolas,'Liberation Mono',monospace;font-size:34px;letter-spacing:10px;font-weight:700;color:#C8AA7C\" aria-label=\"{}\">{}</div>", esc(&k.chars().map(|d| d.to_string()).collect::<Vec<_>>().join(" ")), esc(k))).unwrap_or_default();
    let button = c.button.as_ref().map(|(l, u)| format!("<p style=\"margin:8px 0 20px\"><a href=\"{}\" style=\"display:inline-block;padding:12px 22px;background:#C8AA7C;color:#18191B;text-decoration:none;font-weight:700;border-radius:999px\">{}</a></p>", esc(u), esc(l))).unwrap_or_default();
    let table = if c.table.is_empty() { String::new() } else { format!("<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"margin:0 0 16px;border-collapse:collapse\">{}</table>",
        c.table.iter().map(|(k, v)| format!("<tr><td style=\"padding:8px 0;border-bottom:1px solid #3a3b3f;color:#A4A7AD;font-size:14px\">{}</td><td align=\"right\" style=\"padding:8px 0;border-bottom:1px solid #3a3b3f;color:#F1F1F1;font-size:14px\">{}</td></tr>", esc(k), esc(v))).collect::<String>()) };
    format!("<!doctype html>\n<html lang=\"{lang_attr}\" dir=\"ltr\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><meta name=\"color-scheme\" content=\"dark light\"><meta name=\"supported-color-schemes\" content=\"dark light\"><title>{title}</title>\n<style>@media only screen and (max-width:620px){{.container{{width:100%!important}}.pad{{padding:22px 18px!important}}}}</style></head>\n<body style=\"margin:0;padding:0;background:#18191B;font-family:Inter,-apple-system,'Segoe UI',Roboto,Helvetica,Arial,sans-serif\">\n<div style=\"display:none;max-height:0;overflow:hidden;opacity:0;color:transparent\">{pre}</div>\n<table role=\"presentation\" width=\"100%\" cellpadding=\"0\" cellspacing=\"0\" style=\"background:#18191B\"><tr><td align=\"center\" style=\"padding:24px 12px\">\n<table role=\"presentation\" class=\"container\" width=\"600\" cellpadding=\"0\" cellspacing=\"0\" style=\"width:600px;max-width:100%;background:#222326;border-radius:16px\">\n<tr><td class=\"pad\" style=\"padding:28px 32px 8px\"><div style=\"font-size:15px;letter-spacing:.08em;text-transform:uppercase;color:#C8AA7C;font-weight:700\">{product}</div></td></tr>\n<tr><td class=\"pad\" style=\"padding:8px 32px 8px\"><h1 style=\"margin:0 0 16px;font-size:24px;line-height:1.25;color:#F1F1F1\">{title}</h1>{paras}{code}{button}{table}<p style=\"margin:8px 0 0;font-size:13px;line-height:1.5;color:#A4A7AD\">{note}</p></td></tr>\n<tr><td class=\"pad\" style=\"padding:20px 32px 28px;border-top:1px solid #3a3b3f\"><p style=\"margin:0;font-size:12px;line-height:1.5;color:#A4A7AD\">{footer}<br>{addr}</p></td></tr>\n</table></td></tr></table></body></html>\n",
        title = esc(&c.title), pre = esc(&c.preheader), product = esc(&b.product), note = esc(&c.note), footer = esc(footer_note), addr = esc(&b.legal_address))
}

fn text(c: &Content, b: &Brand) -> String {
    let mut s = format!("{}\n{}\n\n", b.product, c.title);
    for p in &c.paras { s += &format!("{p}\n\n"); }
    if let Some(k) = &c.code { s += &format!("    {k}\n\n"); }
    if let Some((l, u)) = &c.button { s += &format!("{l} : {u}\n\n"); }
    for (k, v) in &c.table { s += &format!("{k} : {v}\n"); }
    if !c.table.is_empty() { s += "\n"; }
    s + &format!("{}\n\n--\n{}\n", c.note, b.legal_address)
}

pub fn render(t: &Template, lang: Lang, to: &str, b: &Brand) -> Message {
    let c = content(t, lang, b);
    Message { to: to.to_string(), subject: c.subject.clone(), html: html(&c, lang, b), text: text(&c, b), kind: kind_of(t) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn all() -> Vec<Template> {
        vec![Template::Verification { name: Some("Élodie".into()), code: "042917".into(), minutes: 10 }, Template::Welcome { name: None }, Template::PasswordRecovery { code: "905531".into(), minutes: 10 },
             Template::Security(SecurityEvent::NewDevice { device: "MacBook".into(), when: "2026-11-28 10:37".into() }), Template::Security(SecurityEvent::PasswordChanged),
             Template::Security(SecurityEvent::TwoFactorEnabled), Template::Security(SecurityEvent::TwoFactorDisabled),
             Template::Security(SecurityEvent::EmailChangeRequested { new_email: "nouvelle@exemple.ca".into() }), Template::Security(SecurityEvent::EmailChanged { new_email: "nouvelle@exemple.ca".into() }), Template::Security(SecurityEvent::AccountExists),
             Template::Billing { invoice_no: "NR-1042".into(), lines: vec![("Licence Studio Pro".into(), 19900)], taxes: vec![("TPS".into(), 995), ("TVQ".into(), 1985)], total: 22880, currency: "CAD".into(), date: (2026, 11, 28) },
             Template::AccountDeletion { effective: (2026, 12, 28) }, Template::EmailChangeNotice { new_email_masked: "n***@exemple.ca".into() }]
    }

    #[test]
    fn every_template_renders_in_both_languages_without_leftovers() {
        for t in all() { for l in [Lang::Fr, Lang::En] {
            let m = render(&t, l, "a@b.co", &Brand::default());
            assert!(m.subject.len() > 8 && m.html.len() > 1000 && m.text.len() > 60, "{t:?} {l:?}");
            for s in [&m.subject, &m.html, &m.text] { assert!(!s.contains("{{") && !s.contains("{code}") && !s.contains("{minutes}") && !s.contains("undefined"), "{t:?}: {s}"); }
            assert!(!m.text.contains('<') && !m.text.contains("&amp;"), "la version texte ne contient pas de HTML");
        } }
    }
    #[test]
    fn languages_differ_and_html_declares_its_language() {
        let t = &all()[0];
        let (fr, en) = (render(t, Lang::Fr, "a@b.co", &Brand::default()), render(t, Lang::En, "a@b.co", &Brand::default()));
        assert!(fr.html.contains("lang=\"fr\"") && en.html.contains("lang=\"en\""));
        assert!(fr.subject.contains("code de vérification") && en.subject.contains("verification code"));
        assert!(fr.text.contains("Bonjour Élodie,") && en.text.contains("Hello Élodie,"));
    }
    #[test]
    fn code_is_present_in_subject_html_and_text_and_nowhere_else() {
        let m = render(&all()[0], Lang::Fr, "a@b.co", &Brand::default());
        assert!(m.subject.contains("042917") && m.html.contains(">042917<") && m.text.contains("042917"));
        let w = render(&all()[1], Lang::Fr, "a@b.co", &Brand::default());
        assert!(!w.html.contains("042917") && !w.subject.contains("042917"));
    }
    #[test]
    fn all_variables_are_html_escaped() {
        let evil = "<script>alert(1)</script>\"'&";
        let t = Template::Verification { name: Some(evil.into()), code: "123456".into(), minutes: 10 };
        let m = render(&t, Lang::En, "a@b.co", &Brand { legal_address: evil.into(), ..Default::default() });
        assert!(!m.html.contains("<script>") && m.html.contains("&lt;script&gt;"));
        let s = render(&Template::Security(SecurityEvent::NewDevice { device: evil.into(), when: evil.into() }), Lang::Fr, "a@b.co", &Brand::default());
        assert!(!s.html.contains("<script>"));
    }
    #[test]
    fn responsive_accessible_and_private_by_construction() {
        for t in all() { let m = render(&t, Lang::Fr, "a@b.co", &Brand::default());
            assert!(m.html.contains("name=\"viewport\"") && m.html.contains("max-width:620px") && m.html.contains("max-width:100%"), "responsive");
            assert!(m.html.contains("role=\"presentation\"") && m.html.contains("<title>") && m.html.contains("<h1"));
            assert!(!m.html.contains("<img") && !m.html.contains("http://") && !m.html.contains("<script"), "aucun pixel de suivi, aucun lien non sécurisé");
            assert!(m.html.len() < 30_000, "taille raisonnable (écrêtage Gmail à 102 Ko)");
        }
    }
    #[test]
    fn brand_colours_from_the_design_system() {
        let m = render(&all()[0], Lang::En, "a@b.co", &Brand::default());
        for c in ["#18191B", "#222326", "#2C2D30", "#F1F1F1", "#A4A7AD", "#C8AA7C"] { assert!(m.html.contains(c), "{c}"); }
    }
    #[test]
    fn billing_uses_locale_formats_and_security_masks_addresses() {
        let b = render(&all()[10], Lang::Fr, "a@b.co", &Brand::default());
        assert!(b.text.contains("228,80\u{A0}$") && b.text.contains("28 novembre 2026") && b.kind == MailKind::Billing, "{}", b.text);
        let be = render(&all()[10], Lang::En, "a@b.co", &Brand::default());
        assert!(be.text.contains("$228.80") && be.text.contains("November 28, 2026"));
        let s = render(&all()[7], Lang::En, "a@b.co", &Brand::default());
        assert!(s.text.contains("n***@exemple.ca") && !s.text.contains("nouvelle@exemple.ca"));
        assert_eq!(mask_email("jean.tremblay@exemple.ca"), "j***@exemple.ca");
        assert_eq!(mask_email("pasdarobase"), "***");
    }
    #[test]
    fn kinds_cover_every_required_category() {
        let kinds: std::collections::BTreeSet<_> = all().iter().map(|t| format!("{:?}", render(t, Lang::En, "a@b.co", &Brand::default()).kind)).collect();
        for k in ["Welcome", "Verification", "PasswordRecovery", "Security", "Billing", "AccountDeletion", "EmailChangeNotice"] { assert!(kinds.contains(k), "{k}"); }
    }
}
