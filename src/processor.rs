use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use std::sync::Arc;

use crate::biquad::Biquad;
use crate::compression::{CompressorSettings, SingleBandCompressor};
use crate::editor;
use crate::params::MultibandCompressorParams;

/// ピークメーターが完全な無音になった後、12dB減衰するのにかかる時間
const PEAK_METER_DECAY_MS: f64 = 150.0;

pub struct MultibandCompressor {
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
    compressors: Vec<[SingleBandCompressor; 3]>,
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
            self.compressors
                .push([SingleBandCompressor::new(), SingleBandCompressor::new(), SingleBandCompressor::new()]);
        }

        // 初期クロスオーバー設定（後述の inherent impl にて実装）
        self.update_crossovers();

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

        let low_settings = CompressorSettings {
            threshold_db: threshold_low,
            ratio: ratio_low,
            attack_coef: attack_coef_low,
            release_coef: release_coef_low,
            makeup_db: makeup_low,
        };

        let mid_settings = CompressorSettings {
            threshold_db: threshold_mid,
            ratio: ratio_mid,
            attack_coef: attack_coef_mid,
            release_coef: release_coef_mid,
            makeup_db: makeup_mid,
        };

        let high_settings = CompressorSettings {
            threshold_db: threshold_high,
            ratio: ratio_high,
            attack_coef: attack_coef_high,
            release_coef: release_coef_high,
            makeup_db: makeup_high,
        };

        // クロスオーバー周波数の更新（頻繁な再初期化を避ける）
        self.update_crossovers();

        let mut peak_amplitude = 0.0_f32;

        for mut channel_samples in buffer.iter_samples() {
            let channel_count = channel_samples.len();
            for ch_idx in 0..channel_count {
                let sample = channel_samples
                    .get_mut(ch_idx)
                    .expect("channel index out of range");
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
                let (low_out, mid_out, high_out) =
                    if let Some(bands) = self.compressors.get_mut(ch_idx) {
                        let low_out = bands[0].process_sample(low, &low_settings);
                        let mid_out = bands[1].process_sample(mid, &mid_settings);
                        let high_out = bands[2].process_sample(high, &high_settings);
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
