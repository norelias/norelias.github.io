//! Assistance contextuelle : relie l'outil survolé ou le défaut détecté aux leçons et exercices.
//! Silencieuse si l'utilisateur a désactivé les aides.

use crate::{exercises, lessons::{self, Lesson}, tooltips};
use norelias_advisor::Kind;
use norelias_onboarding::profile::{Coaching, Profile};

pub struct Context<'a> { pub tool: Option<&'a str>, pub finding: Option<Kind>, /// L'utilisateur a-t-il demandé l'aide (clic sur « ? ») ?
    pub requested: bool }

pub struct Help { pub tooltip: Option<&'static str>, pub lessons: Vec<&'static Lesson>, pub exercise: Option<&'static str> }

fn lesson_for_finding(k: Kind) -> &'static str {
    match k {
        Kind::Exposure => "exposure", Kind::HighlightClipping => "highlights", Kind::WhiteBalance => "white_balance",
        Kind::ColorCast => "tint", Kind::Noise => "noise", Kind::Sharpness => "sharpness", Kind::MotionBlur => "motion_blur",
        Kind::HorizonTilt => "horizon", Kind::Thirds => "thirds",
    }
}

/// `None` si les aides sont désactivées, ou en mode « à la demande » sans demande explicite.
pub fn contextual(profile: &Profile, ctx: &Context) -> Option<Help> {
    if !profile.aids_enabled() { return None; }
    let proactive = profile.coaching == Coaching::Guided;
    if !proactive && !ctx.requested {
        // sans demande explicite, seule l'infobulle de survol (si activée) est montrée
        return ctx.tool.and_then(|t| tooltips::tooltip(profile, t)).map(|t| Help { tooltip: Some(t), lessons: vec![], exercise: None });
    }
    let mut ls: Vec<&Lesson> = vec![];
    if let Some(k) = ctx.finding { if let Some(l) = lessons::get(lesson_for_finding(k)) { ls.push(l); } }
    if let Some(t) = ctx.tool { for l in lessons::LESSONS.iter().filter(|l| l.tools.contains(&t)) { if !ls.iter().any(|x| x.id == l.id) { ls.push(l); } } }
    ls.truncate(3);
    let exercise = ls.iter().find_map(|l| l.exercise).filter(|id| exercises::get(id).is_some());
    Some(Help { tooltip: ctx.tool.and_then(|t| tooltips::tooltip(profile, t)), lessons: ls, exercise })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_finding_kind_maps_to_an_existing_lesson() {
        for k in [Kind::Exposure, Kind::WhiteBalance, Kind::ColorCast, Kind::HighlightClipping, Kind::Noise, Kind::Sharpness,
                  Kind::MotionBlur, Kind::HorizonTilt, Kind::Thirds] {
            assert!(lessons::get(lesson_for_finding(k)).is_some(), "{k:?}");
        }
    }
    #[test]
    fn finding_leads_to_lesson_and_exercise_when_asked() {
        let p = Profile::default();
        let h = contextual(&p, &Context { tool: None, finding: Some(Kind::Exposure), requested: true }).unwrap();
        assert_eq!(h.lessons[0].id, "exposure");
        assert_eq!(h.exercise, Some("ex_exposure"));
    }
    #[test]
    fn on_demand_mode_stays_quiet_until_asked() {
        let p = Profile::default(); // OnDemand par défaut
        let h = contextual(&p, &Context { tool: Some("denoise"), finding: None, requested: false }).unwrap();
        assert!(h.lessons.is_empty() && h.tooltip.is_some());
    }
    #[test]
    fn guided_mode_is_proactive() {
        let mut p = Profile::default();
        p.coaching = Coaching::Guided;
        let h = contextual(&p, &Context { tool: Some("denoise"), finding: None, requested: false }).unwrap();
        assert!(!h.lessons.is_empty());
    }
    #[test]
    fn all_aids_off_means_no_help_at_all() {
        let mut p = Profile::default();
        p.disable_all_aids();
        for requested in [false, true] {
            assert!(contextual(&p, &Context { tool: Some("exposure"), finding: Some(Kind::Exposure), requested }).is_none());
        }
    }
}
