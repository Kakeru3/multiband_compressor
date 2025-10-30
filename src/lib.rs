use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_iced::IcedState;
use std::sync::Arc;

mod editor;
mod biquad;
use crate::biquad::Biquad;

/// ピークメーターが完全な無音になった後、12dB減衰するのにかかる時間
const PEAK_METER_DECAY_MS: f64 = 150.0;

// DSPエンジン用の構造体
struct MultibandCompressor {
    // GUIやホストと共有するパラーメーター
    params: Arc<MultibandCompressorParams>,

    /// ピークメーターが減衰する速さ
    peak_meter_decay_weight: f32,
    // GUIに表示するためのピークメーターの値
    peak_meter: Arc<AtomicF32>,

    // マルチバンド用拡張
    sample_rate: f32,
    // per-channel crossover filters
    filters: Vec<ChannelFilters>,
    // per-channel compressors: [low, mid, high]
    compressors: Vec<[BandCompressor; 3]>,
    current_lo_mid: f32,
    current_mid_hi: f32,
}

struct ChannelFilters {
    low_lp: [Biquad; 2],
    mid_hp: [Biquad; 2],
    mid_lp: [Biquad; 2],
    high_hp: [Biquad; 2],
}

impl ChannelFilters {
    fn new() -> Self {
        Self {
            low_lp: [Biquad::new(), Biquad::new()],
            mid_hp: [Biquad::new(), Biquad::new()],
            mid_lp: [Biquad::new(), Biquad::new()],
            high_hp: [Biquad::new(), Biquad::new()],
        }
    }
}

impl MultibandCompressor {
    // クロスオーバー更新（低域ローパスと高域ハイパス）
    fn update_crossovers(&mut self) {
        let lo_mid = self.params.xover_lo_mid.value();
        let mid_hi = self.params.xover_mid_hi.value();

        let mut needs_update = false;

        if (lo_mid - self.current_lo_mid).abs() > 0.5 {
            self.current_lo_mid = lo_mid;
            needs_update = true;
        }

        if (mid_hi - self.current_mid_hi).abs() > 0.5 {
            self.current_mid_hi = mid_hi;
            needs_update = true;
        }

        if needs_update {
            let nyquist = self.sample_rate * 0.5;
            let low_freq = self.current_lo_mid.clamp(10.0, nyquist * 0.8);
            let high_freq = self.current_mid_hi.clamp(low_freq + 10.0, nyquist * 0.99);

            for filters in self.filters.iter_mut() {
                for lp in filters.low_lp.iter_mut() {
                    lp.set_lowpass(low_freq, self.sample_rate);
                }
                for hp in filters.mid_hp.iter_mut() {
                    hp.set_highpass(low_freq, self.sample_rate);
                }
                for lp in filters.mid_lp.iter_mut() {
                    lp.set_lowpass(high_freq, self.sample_rate);
                }
                for hp in filters.high_hp.iter_mut() {
                    hp.set_highpass(high_freq, self.sample_rate);
                }
            }
        }
    }
}

#[derive(Params)]
struct MultibandCompressorParams {
    #[persist = "editor-state"]
    editor_state: Arc<IcedState>,

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

impl Default for MultibandCompressor {
    fn default() -> Self {
        // Initialize with empty filter/compressor vectors; actual sizes are set in `initialize`
        Self {
            params: Arc::new(MultibandCompressorParams::default()),

            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),

            sample_rate: 44100.0,
            filters: Vec::new(),
            compressors: Vec::new(),
            current_lo_mid: 0.0,
            current_mid_hi: 0.0,
        }
    }
}

impl Default for MultibandCompressorParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

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
                FloatRange::Linear { min: 40.0, max: 1000.0 },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),

            xover_mid_hi: FloatParam::new(
                "Crossover Mid-High",
                2000.0,
                FloatRange::Linear { min: 500.0, max: 8000.0 },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}

// 簡易バンドコンプレッサ（1 チャネル 分の状態）
struct BandCompressor {
    envelope: f32,
    gain_reduction_db: f32,
}

impl BandCompressor {
    fn new() -> Self {
        Self {
            envelope: util::MINUS_INFINITY_DB,
            gain_reduction_db: 0.0,
        }
    }

    fn process_sample(
        &mut self,
        input: f32,
        threshold_db: f32,
        ratio: f32,
        attack_coef: f32,
        release_coef: f32,
        makeup_db: f32,
    ) -> f32 {
        let input_abs = input.abs();
        let input_db = if input_abs > 0.0 { util::gain_to_db(input_abs) } else { util::MINUS_INFINITY_DB };

        if input_db > self.envelope {
            self.envelope = self.envelope * attack_coef + input_db * (1.0 - attack_coef);
        } else {
            self.envelope = self.envelope * release_coef + input_db * (1.0 - release_coef);
        }

        let target_reduction_db = if self.envelope > threshold_db {
            -((self.envelope - threshold_db) * (1.0 - 1.0 / ratio))
        } else {
            0.0_f32
        };

        if target_reduction_db < self.gain_reduction_db {
            self.gain_reduction_db = self.gain_reduction_db * attack_coef + target_reduction_db * (1.0 - attack_coef);
        } else {
            self.gain_reduction_db = self.gain_reduction_db * release_coef + target_reduction_db * (1.0 - release_coef);
        }

        let total_gain = util::db_to_gain(self.gain_reduction_db + makeup_db);
        input * total_gain
    }
}

impl Plugin for MultibandCompressor {
    const NAME: &'static str = "MultibandCompressor GUI (iced)";
    const VENDOR: &'static str = "Kakeru3";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // サンプルレートを保持
        self.sample_rate = buffer_config.sample_rate as f32;

        // チャンネル数に合わせて filters/compressors を (再)構築
        // BufferConfig から直接チャンネル数が得られない場合があるため、とりあえずステレオを仮定して作る。
        // 実際のホストに合わせて必要なら後で動的に再構築してください。
        let ch = 2usize;
        self.current_lo_mid = 0.0;
        self.current_mid_hi = 0.0;
        self.filters.clear();
        self.compressors.clear();
        for _ in 0..ch {
            self.filters.push(ChannelFilters::new());
            self.compressors.push([BandCompressor::new(), BandCompressor::new(), BandCompressor::new()]);
        }

        // 初期クロスオーバー設定（後述の inherent impl にて実装）
        self.update_crossovers();

        // ピークメーターの減衰スピードを、サンプルレートに合わせて設定
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        true
    }

    // update_crossovers is implemented as an inherent method on SimpleCompressor

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Low band parameters
        let threshold_low = self.params.threshold_low.value();
        let ratio_low = self.params.ratio_low.value().max(1.0);
        let attack_low = (self.params.attack_low.value() / 1000.0).max(0.0001);
        let release_low = (self.params.release_low.value() / 1000.0).max(0.0001);
        let makeup_low = self.params.makeup_low.value();

        // Mid band parameters
        let threshold_mid = self.params.threshold_mid.value();
        let ratio_mid = self.params.ratio_mid.value().max(1.0);
        let attack_mid = (self.params.attack_mid.value() / 1000.0).max(0.0001);
        let release_mid = (self.params.release_mid.value() / 1000.0).max(0.0001);
        let makeup_mid = self.params.makeup_mid.value();

        // High band parameters
        let threshold_high = self.params.threshold_high.value();
        let ratio_high = self.params.ratio_high.value().max(1.0);
        let attack_high = (self.params.attack_high.value() / 1000.0).max(0.0001);
        let release_high = (self.params.release_high.value() / 1000.0).max(0.0001);
        let makeup_high = self.params.makeup_high.value();

        // サンプルレートを用いて per-sample coef を計算
        let sample_rate = context.transport().sample_rate as f32;
        let attack_coef_low = (-1.0_f32 / (attack_low * sample_rate)).exp();
        let release_coef_low = (-1.0_f32 / (release_low * sample_rate)).exp();
        let attack_coef_mid = (-1.0_f32 / (attack_mid * sample_rate)).exp();
        let release_coef_mid = (-1.0_f32 / (release_mid * sample_rate)).exp();
        let attack_coef_high = (-1.0_f32 / (attack_high * sample_rate)).exp();
        let release_coef_high = (-1.0_f32 / (release_high * sample_rate)).exp();

        // クロスオーバー周波数の更新（頻繁な再初期化を避ける）
        self.update_crossovers();

        let mut peak_amplitude = 0.0_f32;

        for mut channel_samples in buffer.iter_samples() {
            let channel_count = channel_samples.len();
            for ch_idx in 0..channel_count {
                let sample = channel_samples.get_mut(ch_idx).expect("channel index out of range");
                let input = *sample;

                // 1) バンド分割
                let (low, mid, high) = if let Some(filters) = self.filters.get_mut(ch_idx) {
                    let mut low = input;
                    for biquad in filters.low_lp.iter_mut() {
                        low = biquad.process_sample(low);
                    }

                    let mut high = input;
                    for biquad in filters.high_hp.iter_mut() {
                        high = biquad.process_sample(high);
                    }

                    let mut mid = input;
                    for biquad in filters.mid_hp.iter_mut() {
                        mid = biquad.process_sample(mid);
                    }
                    for biquad in filters.mid_lp.iter_mut() {
                        mid = biquad.process_sample(mid);
                    }

                    (low, mid, high)
                } else {
                    (input, 0.0, 0.0)
                };

                // 2) 各バンドへのコンプレッサー適用
                let (low_out, mid_out, high_out) = if let Some(bands) = self.compressors.get_mut(ch_idx) {
                    let low_out = bands[0].process_sample(
                        low,
                        threshold_low,
                        ratio_low,
                        attack_coef_low,
                        release_coef_low,
                        makeup_low,
                    );
                    let mid_out = bands[1].process_sample(
                        mid,
                        threshold_mid,
                        ratio_mid,
                        attack_coef_mid,
                        release_coef_mid,
                        makeup_mid,
                    );
                    let high_out = bands[2].process_sample(
                        high,
                        threshold_high,
                        ratio_high,
                        attack_coef_high,
                        release_coef_high,
                        makeup_high,
                    );
                    (low_out, mid_out, high_out)
                } else {
                    (low, mid, high)
                };

                let out = low_out + mid_out + high_out;
                *sample = out;

                peak_amplitude = peak_amplitude.max(out.abs());
            }
        }

        // GUI のピークメーター更新
        if self.params.editor_state.is_open() {
            let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
            let new_peak_meter = if peak_amplitude > current_peak_meter {
                peak_amplitude
            } else {
                current_peak_meter * self.peak_meter_decay_weight
                    + peak_amplitude * (1.0 - self.peak_meter_decay_weight)
            };

            self.peak_meter
                .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MultibandCompressor {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain-gui-iced";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for MultibandCompressor {
    const VST3_CLASS_ID: [u8; 16] = *b"CompGuiIcedAaAAa";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(MultibandCompressor);
nih_export_vst3!(MultibandCompressor);