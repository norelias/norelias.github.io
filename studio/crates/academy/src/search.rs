//! Recherche dans les guides : leçons et explications d'outils, insensible à la casse et aux accents.

use crate::{lessons::LESSONS, tooltips};
use norelias_core::Lang;
use norelias_onboarding::tools::TOOLS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitKind { Lesson, Tool }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit { pub kind: HitKind, pub id: &'static str, pub score: u32 }

/// Minuscules + suppression des accents courants du français.
pub fn fold(s: &str) -> String {
    s.chars().flat_map(|c| c.to_lowercase()).map(|c| match c {
        'à' | 'â' | 'ä' | 'á' => 'a', 'é' | 'è' | 'ê' | 'ë' => 'e', 'î' | 'ï' | 'í' => 'i', 'ô' | 'ö' | 'ó' => 'o',
        'ù' | 'û' | 'ü' | 'ú' => 'u', 'ç' => 'c', 'œ' => 'o', '’' => '\'', c => c,
    }).collect()
}

fn tokens(s: &str) -> Vec<String> {
    fold(s).split(|c: char| !c.is_alphanumeric()).filter(|t| t.chars().count() >= 2).map(String::from).collect()
}

/// Tous les mots de la requête doivent apparaître (ET). Titres > mots-clés > texte.
pub fn search(query: &str, lang: Lang) -> Vec<Hit> {
    let q = tokens(query);
    if q.is_empty() { return vec![]; }
    let mut hits = vec![];
    for l in LESSONS {
        let (t, k, b) = (fold(l.title(lang)), fold(l.keywords(lang)), fold(l.body(lang)));
        let mut score = 0;
        let mut all = true;
        for w in &q {
            let s = if t.contains(w.as_str()) { 6 } else if k.contains(w.as_str()) { 3 } else if b.contains(w.as_str()) { 1 } else { 0 };
            if s == 0 { all = false; break; }
            score += s;
        }
        if all { hits.push(Hit { kind: HitKind::Lesson, id: l.id, score }); }
    }
    for t in TOOLS {
        if let Some(txt) = tooltips::text(t.id, lang) {
            let (id, body) = (fold(t.id.replace('_', " ").as_str()), fold(txt));
            let mut score = 0;
            let mut all = true;
            for w in &q {
                let s = if id.contains(w.as_str()) { 6 } else if body.contains(w.as_str()) { 2 } else { 0 };
                if s == 0 { all = false; break; }
                score += s;
            }
            if all { hits.push(Hit { kind: HitKind::Tool, id: t.id, score }); }
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then(a.id.cmp(b.id)));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accent_and_case_insensitive() {
        assert_eq!(fold("Écrêtage Œil ÇA"), "ecretage oil ca");
        let a = search("ecretage", Lang::Fr);
        let b = search("ÉCRÊTAGE", Lang::Fr);
        assert_eq!(a, b);
        assert!(a.iter().any(|h| h.id == "highlights"));
    }
    #[test]
    fn finds_lessons_by_topic_in_both_languages() {
        assert_eq!(search("horizon", Lang::Fr)[0].id, "horizon");
        assert!(search("tripod", Lang::En).iter().any(|h| h.id == "motion_blur"));
        assert!(search("trépied", Lang::Fr).iter().any(|h| h.id == "motion_blur"));
    }
    #[test]
    fn title_matches_rank_first() {
        let r = search("balance des blancs", Lang::Fr);
        assert_eq!(r[0].id, "white_balance");
    }
    #[test]
    fn finds_tools_and_requires_all_words() {
        assert!(search("denoise", Lang::En).iter().any(|h| h.kind == HitKind::Tool && h.id == "denoise"));
        assert!(search("horizon zzzzqq", Lang::En).is_empty());
        assert!(search("   ", Lang::Fr).is_empty());
    }
    #[test]
    fn planned_tools_are_not_returned_as_guides() {
        assert!(search("tone curve", Lang::En).iter().all(|h| h.id != "tone_curve"));
    }
}
