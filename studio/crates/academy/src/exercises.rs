//! Exercices pratiques. Chaque exercice génère une image d'entraînement **procédurale** (libre de droits),
//! applique les opérations de l'élève avec le vrai moteur, puis relance le vrai conseiller :
//! l'exercice est réussi si le défaut ciblé a disparu sans en introduire de plus grave.

use norelias_advisor::{advise, Kind, Severity};
use norelias_core::{ops::apply_stack, synth, Image, Lang, Op};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Training {
    Underexposed, ClippedOverexposed, WarmCast, GreenTint, Noisy, Tilted(f32), CenteredSubject,
}

pub struct Exercise {
    pub id: &'static str,
    pub lesson: &'static str,
    pub training: Training,
    pub target: Kind,
    pub instruction: (&'static str, &'static str),
}

pub const W: usize = 480;
pub const H: usize = 320;

pub static EXERCISES: &[Exercise] = &[
    Exercise { id: "ex_exposure", lesson: "exposure", training: Training::Underexposed, target: Kind::Exposure,
        instruction: ("Cette image est trop sombre. Corrigez l'exposition jusqu'à ce que le conseiller ne signale plus de problème.", "This image is too dark. Adjust the exposure until the advisor no longer reports a problem.") },
    Exercise { id: "ex_highlights", lesson: "highlights", training: Training::ClippedOverexposed, target: Kind::HighlightClipping,
        instruction: ("Les hautes lumières sont écrêtées. Baissez l'exposition et comprimez les hautes lumières.", "Highlights are clipped. Lower the exposure and compress the highlights.") },
    Exercise { id: "ex_white_balance", lesson: "white_balance", training: Training::WarmCast, target: Kind::WhiteBalance,
        instruction: ("L'image est trop chaude. Rééquilibrez les canaux rouge et bleu.", "The image is too warm. Rebalance the red and blue channels.") },
    Exercise { id: "ex_tint", lesson: "tint", training: Training::GreenTint, target: Kind::ColorCast,
        instruction: ("Les gris tirent vers le vert. Compensez le canal vert.", "The greys lean green. Compensate the green channel.") },
    Exercise { id: "ex_noise", lesson: "noise", training: Training::Noisy, target: Kind::Noise,
        instruction: ("Cette image est bruyante. Réduisez le bruit sans effacer les contours.", "This image is noisy. Reduce the noise without erasing the edges.") },
    Exercise { id: "ex_horizon", lesson: "horizon", training: Training::Tilted(3.5), target: Kind::HorizonTilt,
        instruction: ("L'horizon penche. Redressez l'image.", "The horizon is tilted. Straighten the image.") },
    Exercise { id: "ex_thirds", lesson: "thirds", training: Training::CenteredSubject, target: Kind::Thirds,
        instruction: ("Le sujet est au centre. Recadrez pour le placer sur un point fort des tiers.", "The subject is in the centre. Crop to place it on a thirds intersection.") },
];

impl Exercise {
    pub fn instruction(&self, l: Lang) -> &'static str { if l == Lang::Fr { self.instruction.0 } else { self.instruction.1 } }
    /// Photographie d'entraînement (déterministe).
    pub fn image(&self) -> Image { training_image(self.training) }
}

pub fn get(id: &str) -> Option<&'static Exercise> { EXERCISES.iter().find(|e| e.id == id) }

pub fn training_image(t: Training) -> Image {
    let base = synth::scene(W, H, 0.0);
    match t {
        Training::Underexposed => synth::exposure(&base, -2.0),
        Training::ClippedOverexposed => synth::clip(&synth::exposure(&base, 2.5)),
        Training::WarmCast => synth::tint(&base, [1.3, 1.0, 0.75]),
        Training::GreenTint => synth::tint(&base, [1.0, 1.35, 1.0]),
        Training::Noisy => synth::add_noise(&base, 0.04, 42),
        Training::Tilted(d) => synth::scene(W, H, d),
        Training::CenteredSubject => synth::centered_subject(W, H),
    }
}

#[derive(Debug)]
pub struct Outcome {
    pub passed: bool,
    pub target_still_present: bool,
    /// Défauts graves apparus à cause des réglages de l'élève.
    pub introduced: Vec<Kind>,
}

/// Vérifie les opérations de l'élève avec le vrai moteur.
pub fn check(ex: &Exercise, ops: &[Op]) -> Outcome {
    let src = ex.image();
    let before = advise(&src);
    let after = advise(&apply_stack(&src, ops));
    let target_still_present = after.has(ex.target);
    let introduced: Vec<Kind> = after.findings.iter()
        .filter(|f| f.severity == Severity::Strong && !before.has(f.kind)).map(|f| f.kind).collect();
    Outcome { passed: !target_still_present && introduced.is_empty(), target_still_present, introduced }
}

#[cfg(test)]
mod tests {
    use super::*;
    use norelias_advisor::apply_selected;

    #[test]
    fn every_exercise_references_an_existing_lesson_and_lesson_links_back() {
        for e in EXERCISES {
            let l = crate::lessons::get(e.lesson).unwrap_or_else(|| panic!("leçon manquante {}", e.lesson));
            assert_eq!(l.exercise, Some(e.id));
        }
        for l in crate::lessons::LESSONS { if let Some(id) = l.exercise { assert!(get(id).is_some(), "exercice manquant {id}"); } }
    }

    #[test]
    fn doing_nothing_never_passes() {
        for e in EXERCISES {
            let o = check(e, &[]);
            assert!(!o.passed && o.target_still_present, "{} réussi sans rien faire : le défaut n'est pas détecté", e.id);
        }
    }

    #[test]
    fn solving_with_advisor_suggestion_passes_every_exercise() {
        for e in EXERCISES {
            let img = e.image();
            let r = advise(&img);
            let f = r.find(e.target).unwrap_or_else(|| panic!("{} : défaut ciblé non détecté", e.id));
            let fixed = apply_selected(&img, &[f]);
            assert!(fixed.width > 0);
            let o = check(e, &f.ops);
            assert!(o.passed, "{} : la correction du conseiller doit réussir : {o:?}", e.id);
        }
    }

    #[test]
    fn wrong_answers_fail() {
        let ex = get("ex_exposure").unwrap();
        assert!(!check(ex, &[Op::Exposure { ev: 0.3 }]).passed, "correction insuffisante");
        let o = check(ex, &[Op::Exposure { ev: 6.0 }]);
        assert!(!o.passed, "surcorrection");
        let wb = get("ex_white_balance").unwrap();
        assert!(!check(wb, &[Op::Exposure { ev: 0.5 }]).passed, "un réglage sans rapport ne doit pas réussir");
    }

    #[test]
    fn training_images_are_deterministic() {
        for e in EXERCISES { assert_eq!(e.image(), e.image()); }
    }

    #[test]
    fn instructions_in_both_languages() {
        for e in EXERCISES { for l in [Lang::Fr, Lang::En] { assert!(e.instruction(l).len() > 20); } }
    }
}
