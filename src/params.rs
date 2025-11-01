use nih_plug::prelude::*;
use nih_plug_iced::IcedState;
use std::sync::Arc;

#[derive(Params)]
pub struct MultibandCompressorParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<IcedState>,

    // Low band parameters
    #[id = "threshold_low"]
    pub threshold_low: FloatParam,
    #[id = "ratio_low"]
    pub ratio_low: FloatParam,
    #[id = "attack_low"]
    pub attack_low: FloatParam,
    #[id = "release_low"]
    pub release_low: FloatParam,
    #[id = "makeup_low"]
    pub makeup_low: FloatParam,

    // Mid band parameters
    #[id = "threshold_mid"]
    pub threshold_mid: FloatParam,
    #[id = "ratio_mid"]
    pub ratio_mid: FloatParam,
    #[id = "attack_mid"]
    pub attack_mid: FloatParam,
    #[id = "release_mid"]
    pub release_mid: FloatParam,
    #[id = "makeup_mid"]
    pub makeup_mid: FloatParam,

    // High band parameters
    #[id = "threshold_high"]
    pub threshold_high: FloatParam,
    #[id = "ratio_high"]
    pub ratio_high: FloatParam,
    #[id = "attack_high"]
    pub attack_high: FloatParam,
    #[id = "release_high"]
    pub release_high: FloatParam,
    #[id = "makeup_high"]
    pub makeup_high: FloatParam,

    // Crossover frequencies
    #[id = "xover_lo_mid"]
    pub xover_lo_mid: FloatParam,
    #[id = "xover_mid_hi"]
    pub xover_mid_hi: FloatParam,
}

impl Default for MultibandCompressorParams {
    fn default() -> Self {
        Self {
            editor_state: IcedState::from_size(680, 500),

            // Low band
            threshold_low: FloatParam::new(
                "Threshold Low",
                -12.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            ratio_low: FloatParam::new(
                "Ratio Low",
                2.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            attack_low: FloatParam::new(
                "Attack Low",
                20.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 100.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            release_low: FloatParam::new(
                "Release Low",
                150.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 1000.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            makeup_low: FloatParam::new(
                "Makeup Low",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // Mid band
            threshold_mid: FloatParam::new(
                "Threshold Mid",
                -10.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            ratio_mid: FloatParam::new(
                "Ratio Mid",
                3.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            attack_mid: FloatParam::new(
                "Attack Mid",
                10.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 100.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            release_mid: FloatParam::new(
                "Release Mid",
                100.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 1000.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            makeup_mid: FloatParam::new(
                "Makeup Mid",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // High band
            threshold_high: FloatParam::new(
                "Threshold High",
                -8.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            ratio_high: FloatParam::new(
                "Ratio High",
                4.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            attack_high: FloatParam::new(
                "Attack High",
                5.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 100.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            release_high: FloatParam::new(
                "Release High",
                80.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 1000.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            makeup_high: FloatParam::new(
                "Makeup High",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            // Crossovers
            xover_lo_mid: FloatParam::new(
                "Crossover Low-Mid",
                200.0,
                FloatRange::Linear {
                    min: 40.0,
                    max: 1000.0,
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            xover_mid_hi: FloatParam::new(
                "Crossover Mid-High",
                2000.0,
                FloatRange::Linear {
                    min: 500.0,
                    max: 8000.0,
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}
