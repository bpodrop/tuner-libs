pub fn compute_nsdf(frame: &[f32], tau_min: usize, tau_max: usize) -> Vec<f32> {
    if frame.is_empty() || tau_min > tau_max || tau_max >= frame.len() {
        return Vec::new();
    }

    let mut values = Vec::with_capacity(tau_max - tau_min + 1);

    for tau in tau_min..=tau_max {
        let mut acf = 0.0_f32;
        let mut energy = 0.0_f32;

        for index in 0..(frame.len() - tau) {
            let x = frame[index];
            let y = frame[index + tau];
            acf += x * y;
            energy += x * x + y * y;
        }

        let value = if energy > 0.0 {
            (2.0 * acf) / energy
        } else {
            0.0
        };
        values.push(value);
    }

    values
}
