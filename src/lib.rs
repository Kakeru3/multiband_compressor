use nih_plug::prelude::*;

mod biquad;
mod compression;
mod editor;
mod params;
mod processor;

pub use params::MultibandCompressorParams;
pub use processor::MultibandCompressor;

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
