use nih_plug::prelude::util;

/// 少なくとも 1 バンド分のコンプレッション状態を保持するシンプルなコンプレッサー。
#[derive(Debug, Clone)]
pub struct SingleBandCompressor {
    envelope: f32,
    gain_reduction_db: f32,
}

impl SingleBandCompressor {
    pub fn new() -> Self {
        Self {
            envelope: util::MINUS_INFINITY_DB,
            gain_reduction_db: 0.0,
        }
    }

    pub fn process_sample(&mut self, input: f32, settings: &CompressorSettings) -> f32 {
        let input_abs = input.abs();
        let input_db = if input_abs > 0.0 {
            util::gain_to_db(input_abs)
        } else {
            util::MINUS_INFINITY_DB
        };

        if input_db > self.envelope {
            self.envelope =
                self.envelope * settings.attack_coef + input_db * (1.0 - settings.attack_coef);
        } else {
            self.envelope =
                self.envelope * settings.release_coef + input_db * (1.0 - settings.release_coef);
        }

        let target_reduction_db = if self.envelope > settings.threshold_db {
            -((self.envelope - settings.threshold_db) * (1.0 - 1.0 / settings.ratio.max(1.0)))
        } else {
            0.0_f32
        };

        if target_reduction_db < self.gain_reduction_db {
            self.gain_reduction_db = self.gain_reduction_db * settings.attack_coef
                + target_reduction_db * (1.0 - settings.attack_coef);
        } else {
            self.gain_reduction_db = self.gain_reduction_db * settings.release_coef
                + target_reduction_db * (1.0 - settings.release_coef);
        }

        let total_gain = util::db_to_gain(self.gain_reduction_db + settings.makeup_db);
        input * total_gain
    }
}

impl Default for SingleBandCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompressorSettings {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_coef: f32,
    pub release_coef: f32,
    pub makeup_db: f32,
}
