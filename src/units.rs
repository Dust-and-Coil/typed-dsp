use core::ops::{Add, Sub};

/// A signal's absolute amplitude, linear and normalized to full scale:
/// 1.0 is 0 dBFS, and floating-point signals may go above it.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PeakAmplitude(f32);

/// A signal level relative to digital full scale. Silence is negative
/// infinity.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct LevelDbfs(f32);

/// A multiplier applied to a signal.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct LinearGain(f32);

/// A gain in decibels. Negative infinity is the gain that silences.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct GainDb(f32);

impl PeakAmplitude {
    pub const SILENCE: Self = Self(0.0);

    pub const fn new(amplitude: f32) -> Option<Self> {
        if is_magnitude(amplitude) {
            Some(Self(amplitude))
        } else {
            None
        }
    }

    /// A host sample's amplitude, or None for a non-finite sample.
    pub fn from_sample(sample: f32) -> Option<Self> {
        Self::new(sample.abs())
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn to_dbfs(self) -> LevelDbfs {
        LevelDbfs(amplitude_to_db(self.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl LevelDbfs {
    pub const SILENCE: Self = Self(f32::NEG_INFINITY);

    pub const fn new(dbfs: f32) -> Option<Self> {
        if is_decibels(dbfs) {
            Some(Self(dbfs))
        } else {
            None
        }
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn to_amplitude(self) -> PeakAmplitude {
        PeakAmplitude(db_to_amplitude(self.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl LinearGain {
    pub const SILENCE: Self = Self(0.0);
    pub const UNITY:   Self = Self(1.0);

    pub const fn new(multiplier: f32) -> Option<Self> {
        if is_magnitude(multiplier) {
            Some(Self(multiplier))
        } else {
            None
        }
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn to_db(self) -> GainDb {
        GainDb(amplitude_to_db(self.0))
    }
}

impl GainDb {
    pub const ZERO: Self = Self(0.0);
    pub const MUTE: Self = Self(f32::NEG_INFINITY);

    pub const fn new(db: f32) -> Option<Self> {
        if is_decibels(db) {
            Some(Self(db))
        } else {
            None
        }
    }

    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn to_linear(self) -> LinearGain {
        LinearGain(db_to_amplitude(self.0))
    }

    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self(self.0.clamp(min.0, max.0))
    }
}

/// The gain that takes `other` to `self`.
impl Sub for LevelDbfs {
    type Output = GainDb;

    fn sub(self, other: Self) -> GainDb {
        GainDb(decibel_difference(self.0, other.0))
    }
}

impl Add<GainDb> for LevelDbfs {
    type Output = Self;

    fn add(self, gain: GainDb) -> Self {
        Self(decibel_sum(self.0, gain.0))
    }
}

impl Add for GainDb {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(decibel_sum(self.0, other.0))
    }
}

const fn is_magnitude(x: f32) -> bool {
    x.is_finite() && x >= 0.0
}

const fn is_decibels(x: f32) -> bool {
    !x.is_nan() && x != f32::INFINITY
}

/// Neither term is NaN or positive infinity, so the sum is not NaN; only
/// overflow needs capping.
fn decibel_sum(a: f32, b: f32) -> f32 {
    (a + b).min(f32::MAX)
}

/// Equal values, silence included, differ by nothing; anything above
/// silence is the largest finite gain above it.
fn decibel_difference(a: f32, b: f32) -> f32 {
    if a == b {
        0.0
    } else {
        (a - b).min(f32::MAX)
    }
}

fn amplitude_to_db(amplitude: f32) -> f32 {
    20.0 * libm::log10f(amplitude)
}

fn db_to_amplitude(db: f32) -> f32 {
    libm::powf(10.0, db * 0.05).min(f32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HALF:    PeakAmplitude = PeakAmplitude::new(0.5).unwrap();
    const QUIET:   LevelDbfs     = LevelDbfs::new(-10.0).unwrap();
    const BOOST:   GainDb        = GainDb::new(6.0).unwrap();
    const CUT:     GainDb        = GainDb::new(-6.0).unwrap();

    fn level(dbfs: f32) -> LevelDbfs {
        LevelDbfs::new(dbfs).unwrap()
    }

    fn gain_db(db: f32) -> GainDb {
        GainDb::new(db).unwrap()
    }

    #[test]
    fn magnitudes_must_be_finite_and_non_negative() {
        let invalid = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.25];
        assert!(invalid.iter().all(|&x| PeakAmplitude::new(x).is_none() && LinearGain::new(x).is_none()));
    }

    #[test]
    fn magnitudes_may_exceed_full_scale() {
        assert_eq!((PeakAmplitude::new(1.5).map(PeakAmplitude::get), LinearGain::new(4.0).map(LinearGain::get)), (Some(1.5), Some(4.0)));
    }

    #[test]
    fn decibels_reject_nan_and_positive_infinity() {
        let invalid = [f32::NAN, f32::INFINITY];
        assert!(invalid.iter().all(|&x| LevelDbfs::new(x).is_none() && GainDb::new(x).is_none()));
    }

    #[test]
    fn decibels_accept_negative_infinity_as_silence() {
        assert_eq!((level(f32::NEG_INFINITY), GainDb::new(f32::NEG_INFINITY)), (LevelDbfs::SILENCE, Some(GainDb::MUTE)));
    }

    #[test]
    fn constants_are_built_at_compile_time() {
        assert_eq!((HALF.get(), QUIET.get(), BOOST.get()), (0.5, -10.0, 6.0));
    }

    #[test]
    fn a_sample_amplitude_is_its_magnitude() {
        let amplitudes = [-0.5, 0.0, 1.5].map(|x| PeakAmplitude::from_sample(x).map(PeakAmplitude::get));
        assert_eq!(amplitudes, [Some(0.5), Some(0.0), Some(1.5)]);
    }

    #[test]
    fn non_finite_samples_have_no_amplitude() {
        let amplitudes = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY].map(PeakAmplitude::from_sample);
        assert_eq!(amplitudes, [None; 3]);
    }

    #[test]
    fn the_louder_peak_wins() {
        assert_eq!(PeakAmplitude::SILENCE.max(HALF), HALF);
    }

    #[test]
    fn full_scale_is_zero_dbfs() {
        assert_eq!(PeakAmplitude::new(1.0).unwrap().to_dbfs(), level(0.0));
    }

    #[test]
    fn half_amplitude_is_about_minus_six_dbfs() {
        assert!((HALF.to_dbfs().get() + 6.0206).abs() < 1.0e-3);
    }

    #[test]
    fn silent_amplitude_is_silent_level_and_back() {
        let level = PeakAmplitude::SILENCE.to_dbfs();
        assert_eq!((level, level.to_amplitude()), (LevelDbfs::SILENCE, PeakAmplitude::SILENCE));
    }

    #[test]
    fn the_loudest_amplitude_has_a_finite_level() {
        assert!(PeakAmplitude::new(f32::MAX).unwrap().to_dbfs().get().is_finite());
    }

    #[test]
    fn the_loudest_level_has_a_finite_amplitude() {
        assert_eq!(level(f32::MAX).to_amplitude().get(), f32::MAX);
    }

    #[test]
    fn the_louder_level_wins() {
        assert_eq!(LevelDbfs::SILENCE.max(QUIET), QUIET);
    }

    #[test]
    fn zero_db_is_exactly_unity_gain_and_back() {
        assert_eq!((GainDb::ZERO.to_linear(), LinearGain::UNITY.to_db()), (LinearGain::UNITY, GainDb::ZERO));
    }

    #[test]
    fn muting_gain_is_silent_multiplier_and_back() {
        assert_eq!((GainDb::MUTE.to_linear(), LinearGain::SILENCE.to_db()), (LinearGain::SILENCE, GainDb::MUTE));
    }

    #[test]
    fn any_gain_db_converts_to_a_finite_multiplier() {
        assert_eq!(gain_db(f32::MAX).to_linear().get(), f32::MAX);
    }

    #[test]
    fn clamping_a_gain_keeps_it_in_range() {
        assert_eq!((GainDb::MUTE.clamp(CUT, BOOST), gain_db(f32::MAX).clamp(CUT, BOOST)), (CUT, BOOST));
    }

    #[test]
    fn the_difference_of_two_levels_is_a_gain() {
        assert_eq!(level(-10.0) - level(-4.0), CUT);
    }

    #[test]
    fn silence_less_silence_is_zero_gain() {
        assert_eq!(LevelDbfs::SILENCE - LevelDbfs::SILENCE, GainDb::ZERO);
    }

    #[test]
    fn silence_less_any_level_is_muting_gain() {
        assert_eq!(LevelDbfs::SILENCE - QUIET, GainDb::MUTE);
    }

    #[test]
    fn any_level_less_silence_is_the_largest_finite_gain() {
        assert_eq!((QUIET - LevelDbfs::SILENCE).get(), f32::MAX);
    }

    #[test]
    fn a_gain_moves_a_level() {
        assert_eq!(QUIET + BOOST, level(-4.0));
    }

    #[test]
    fn silence_stays_silent_under_any_gain() {
        assert_eq!((LevelDbfs::SILENCE + BOOST, LevelDbfs::SILENCE + GainDb::MUTE), (LevelDbfs::SILENCE, LevelDbfs::SILENCE));
    }

    #[test]
    fn muting_gain_silences_any_level() {
        assert_eq!(QUIET + GainDb::MUTE, LevelDbfs::SILENCE);
    }

    #[test]
    fn gains_add() {
        assert_eq!((BOOST + CUT, BOOST + BOOST), (GainDb::ZERO, gain_db(12.0)));
    }

    #[test]
    fn muting_gain_absorbs_any_gain() {
        assert_eq!((GainDb::MUTE + BOOST, GainDb::MUTE + GainDb::MUTE), (GainDb::MUTE, GainDb::MUTE));
    }

    #[test]
    fn decibel_arithmetic_saturates_at_the_largest_finite_value() {
        let most = gain_db(f32::MAX);
        assert_eq!(((most + most).get(), (level(f32::MAX) - level(f32::MIN)).get(), (level(f32::MAX) + most).get()), (f32::MAX, f32::MAX, f32::MAX));
    }
}
