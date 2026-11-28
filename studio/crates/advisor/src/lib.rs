//! Conseiller photographique : mesure des caractéristiques techniques d'une image,
//! détection de problèmes, explications bilingues et corrections **prévisualisables**.
//!
//! Principes :
//! * le conseiller ne modifie jamais l'image source ;
//! * chaque suggestion contient des `Op` réelles du moteur, appliquées via `preview`/`apply` ;
//! * l'utilisateur choisit quelles suggestions appliquer (`apply_selected`) ;
//!   l'application automatique (`auto_apply`) n'inclut que les corrections « sûres » (`auto_safe`),
//!   jamais les choix créatifs (recadrage).
//!
//! Ce sont des heuristiques : un contre-jour voulu ou une image « high-key » peuvent être signalés.
//! Les messages le disent, et rien n'est appliqué sans décision (manuelle ou automatique explicite).

mod analysis;
mod text;

pub use analysis::{analyze, Metrics, Thresholds};
pub use norelias_core::Lang;
use norelias_core::{ops::apply_stack, Image, Op};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Exposure,
    WhiteBalance,
    ColorCast,
    HighlightClipping,
    Noise,
    Sharpness,
    MotionBlur,
    HorizonTilt,
    Thirds,
}

impl Kind {
    pub const ALL: [Kind; 9] = [Kind::Exposure, Kind::WhiteBalance, Kind::ColorCast, Kind::HighlightClipping, Kind::Noise,
        Kind::Sharpness, Kind::MotionBlur, Kind::HorizonTilt, Kind::Thirds];
    pub fn from_id(id: &str) -> Option<Kind> { Kind::ALL.into_iter().find(|k| k.id() == id) }
    pub fn id(self) -> &'static str {
        match self {
            Kind::Exposure => "exposure",
            Kind::WhiteBalance => "white_balance",
            Kind::ColorCast => "color_cast",
            Kind::HighlightClipping => "highlight_clipping",
            Kind::Noise => "noise",
            Kind::Sharpness => "sharpness",
            Kind::MotionBlur => "motion_blur",
            Kind::HorizonTilt => "horizon_tilt",
            Kind::Thirds => "thirds",
        }
    }
    /// Ordre canonique d'application des corrections.
    fn stage(self) -> u8 {
        match self {
            Kind::Exposure => 0,
            Kind::WhiteBalance | Kind::ColorCast => 1,
            Kind::HighlightClipping => 2,
            Kind::Noise => 3,
            Kind::Sharpness => 4,
            Kind::HorizonTilt => 5,
            Kind::Thirds => 6,
            Kind::MotionBlur => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Warning,
    Strong,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub kind: Kind,
    pub severity: Severity,
    /// Valeur mesurée principale (unité dépendant du type : IL, fraction, σ, degrés…).
    pub value: f32,
    /// Corrections proposées (peut être vide si aucune correction logicielle honnête n'existe).
    pub ops: Vec<Op>,
    /// Appliquée par `auto_apply` ? Faux pour les décisions créatives ou incertaines.
    pub auto_safe: bool,
}

impl Finding {
    pub fn title(&self, lang: Lang) -> String { text::title(self, lang) }
    pub fn explain(&self, lang: Lang) -> String { text::explain(self, lang) }
    /// Aperçu rapide : image réduite à `max_side` puis corrections appliquées.
    pub fn preview(&self, src: &Image, max_side: usize) -> Image {
        apply_stack(&src.downscale_to(max_side), &self.ops)
    }
}

pub struct Report {
    pub metrics: Metrics,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn find(&self, k: Kind) -> Option<&Finding> { self.findings.iter().find(|f| f.kind == k) }
    pub fn has(&self, k: Kind) -> bool { self.find(k).is_some() }
}

/// Analyse avec les seuils par défaut.
pub fn advise(img: &Image) -> Report {
    advise_with(img, &Thresholds::default())
}

pub fn advise_with(img: &Image, th: &Thresholds) -> Report {
    let metrics = analyze(img, th);
    let findings = analysis::findings(&metrics, th);
    Report { metrics, findings }
}

/// Applique les suggestions choisies par l'utilisateur, dans l'ordre canonique.
pub fn apply_selected(src: &Image, findings: &[&Finding]) -> Image {
    let mut sel: Vec<&&Finding> = findings.iter().collect();
    sel.sort_by_key(|f| f.kind.stage());
    let ops: Vec<Op> = sel.iter().flat_map(|f| f.ops.clone()).collect();
    apply_stack(src, &ops)
}

/// Application automatique : uniquement les corrections `auto_safe`. Retourne aussi la liste appliquée.
pub fn auto_apply(src: &Image, report: &Report) -> (Image, Vec<Kind>) {
    let chosen: Vec<&Finding> = report.findings.iter().filter(|f| f.auto_safe && !f.ops.is_empty()).collect();
    let kinds = chosen.iter().map(|f| f.kind).collect();
    (apply_selected(src, &chosen), kinds)
}
