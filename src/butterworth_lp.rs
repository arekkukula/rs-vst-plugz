use std::f32::consts::PI;
/// Butterworth's LP filter implementation, based on:
/// https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
pub fn lowpass_filter(data: &mut [f32], sample_rate: f32, cutoff_frequency: f32) {
    let dt = 1.0 / sample_rate;
    let rc = 1.0 / (cutoff_frequency * 2.0 * PI);
    let alpha = dt / (rc + dt);

    // The first sample has to be calculated manually because the algorithm
    // relies on always having a previous value.
    data[0] *= alpha;

    for i in 1..data.len() {
        data[i] = data[i - 1] + alpha * (data[i] - data[i - 1]);
    }
}

pub fn lowpass_two_samples(
    current: f32,
    previous: f32,
    sample_rate: f32,
    cutoff_frequency: f32,
) -> f32 {
    let dt = 1.0 / sample_rate;
    let rc = 1.0 / (cutoff_frequency * 2.0 * PI);
    let alpha = dt / (rc + dt);
    previous + alpha * (current - previous)
}
