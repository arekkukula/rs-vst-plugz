/// A simple gain stage. Expects passing value in dB, e.g. by using
/// `nih_plug::util::db_to_gain`
pub fn makeup(sample: &mut f32, db: f32) {
    *sample *= db;
}
