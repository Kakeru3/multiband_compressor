#[derive(Clone, Copy)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn new() -> Self {
        Self { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z1: 0.0, z2: 0.0 }
    }

    pub fn process_sample(&mut self, x: f32) -> f32 {
        // Direct Form II Transposed to keep numerical stability
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn set_lowpass(&mut self, freq: f32, sr: f32) {
        // 2nd-order Butterworth (approximate)
        let omega = 2.0 * std::f32::consts::PI * freq / sr;
        let cosw = omega.cos();
        let sinw = omega.sin();
        let q = 1.0 / 2f32.sqrt();
        let alpha = sinw / (2.0 * q);
        let b0 = (1.0 - cosw) / 2.0;
        let b1 = 1.0 - cosw;
        let b2 = (1.0 - cosw) / 2.0;
        let a0 = 1.0 + alpha;
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = -2.0 * cosw / a0;
        self.a2 = (1.0 - alpha) / a0;
        // reset states to avoid clicks on coefficient change
        self.z1 = 0.0;
        self.z2 = 0.0;
    }

    pub fn set_highpass(&mut self, freq: f32, sr: f32) {
        let omega = 2.0 * std::f32::consts::PI * freq / sr;
        let cosw = omega.cos();
        let sinw = omega.sin();
        let q = 1.0 / 2f32.sqrt();
        let alpha = sinw / (2.0 * q);
        let b0 = (1.0 + cosw) / 2.0;
        let b1 = -(1.0 + cosw);
        let b2 = (1.0 + cosw) / 2.0;
        let a0 = 1.0 + alpha;
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = -2.0 * cosw / a0;
        self.a2 = (1.0 - alpha) / a0;
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}
