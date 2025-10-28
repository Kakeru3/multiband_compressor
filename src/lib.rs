use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_iced::IcedState;
use std::sync::Arc;

mod editor;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 150.0;

// DSPエンジン用の構造体
struct SimpleCompressor {
    // GUIやホストと共有するパラーメーター
    params: Arc<SimpleCompressorParams>,

    /// ピークメーターが減衰する速さ
    peak_meter_decay_weight: f32,
    // GUIに表示するためのピークメーターの値

    peak_meter: Arc<AtomicF32>,
    /// 入力信号のレベルを追従する値
    envelope: f32,
    /// 現在のゲインリダクション
    gain_reduction_db: f32,
}

#[derive(Params)]
struct SimpleCompressorParams {
    #[persist = "editor-state"]
    editor_state: Arc<IcedState>,

    #[id = "threshold"]
    pub threshold: FloatParam,

    #[id = "ratio"]
    pub ratio: FloatParam,

    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "release"]
    pub release: FloatParam,

    #[id = "makeup"]
    pub makeup: FloatParam,
}

impl Default for SimpleCompressor {
    fn default() -> Self {
        Self {
            params: Arc::new(SimpleCompressorParams::default()),

            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            envelope: util::MINUS_INFINITY_DB,
            gain_reduction_db: 0.0,
        }
    }
}

impl Default for SimpleCompressorParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            threshold: FloatParam::new(
                "Threshold",
                0.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            ratio: FloatParam::new(
                "Ratio",
                2.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            attack: FloatParam::new(
                "Attack",
                10.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 100.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 1000.0,
                },
            )
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            makeup: FloatParam::new(
                "Makeup",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for SimpleCompressor {
    const NAME: &'static str = "SimpleCompressor GUI (iced)";
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
        // ピークメーターの減衰スピードを、サンプルレートに合わせて設定
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // guiからパラメーターを取得
        let threshold_db = self.params.threshold.smoothed.next();
        let ratio = self.params.ratio.smoothed.next().max(1.0);
        let attack_time = (self.params.attack.smoothed.next() / 1000.0).max(0.0001); // seconds
        let release_time = (self.params.release.smoothed.next() / 1000.0).max(0.0001); // seconds
        let makeup_db = self.params.makeup.smoothed.next();

        // f32のサンプルレート
        let sample_rate = context.transport().sample_rate as f32;

        for channel_samples in buffer.iter_samples() {
            let mut amplitude = 0.0_f32;
            let num_samples = channel_samples.len() as f32;

            // スムージング係数の計算(歪まないようにアタックとリリースの計算をしているところ)
            let attack_coef_per_sample = (-1.0_f32 / (attack_time * sample_rate)).exp();
            let release_coef_per_sample = (-1.0_f32 / (release_time * sample_rate)).exp();

            // 1サンプルずつのループ
            for sample in channel_samples {
                // 絶対値を代入しているので、波形の正負ではなく、振幅のみを扱っている
                let input = sample.abs();

                // 入力レベルを、dbに変換
                let input_db = if input > 0.0 { util::gain_to_db(input) } else { util::MINUS_INFINITY_DB };

                // コンプレッサーの心臓(急に音量が上がったら、attack速度で追従、下がったら、Release速度で追従)
                if input_db > self.envelope {
                    self.envelope = self.envelope * attack_coef_per_sample + input_db * (1.0 - attack_coef_per_sample);
                } else {
                    self.envelope = self.envelope * release_coef_per_sample + input_db * (1.0 - release_coef_per_sample);
                }

                // ratioの値を使って、ゲインリダクションを計算
                let target_reduction_db = if self.envelope > threshold_db {
                    -((self.envelope - threshold_db) * (1.0 - 1.0 / ratio))
                } else {
                    0.0_f32
                };

                // target_reduction_dbに代入された値を見て、かかり具合を調整
                if target_reduction_db < self.gain_reduction_db {
                    self.gain_reduction_db = self.gain_reduction_db * attack_coef_per_sample + target_reduction_db * (1.0 - attack_coef_per_sample);
                } else {
                    self.gain_reduction_db = self.gain_reduction_db * release_coef_per_sample + target_reduction_db * (1.0 - release_coef_per_sample);
                }

                // db_to_gain(x) は 10^(x / 20)。減衰dB + メイクアップdB → 総ゲインに変換
                // 最後の += sample.abs()で、出力音量を調整してる
                let total_gain = util::db_to_gain(self.gain_reduction_db + makeup_db);
                *sample *= total_gain;

                amplitude += sample.abs();
            }

            // guiのピークメーターの更新
            if self.params.editor_state.is_open() {
                amplitude = amplitude / num_samples;
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed);
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SimpleCompressor {
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

impl Vst3Plugin for SimpleCompressor {
    const VST3_CLASS_ID: [u8; 16] = *b"CompGuiIcedAaAAa";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(SimpleCompressor);
nih_export_vst3!(SimpleCompressor);