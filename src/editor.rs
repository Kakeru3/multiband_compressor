use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor, GuiContext};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;
use std::time::Duration;

use crate::MultibandCompressorParams;

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(680, 500)
}

pub(crate) fn create(
    params: Arc<MultibandCompressorParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<MultibandCompressorEditor>(editor_state, (params, peak_meter))
}

struct MultibandCompressorEditor {
    params: Arc<MultibandCompressorParams>,
    context: Arc<dyn GuiContext>,

    peak_meter: Arc<AtomicF32>,

    // Low band sliders
    threshold_low_slider_state: nih_widgets::param_slider::State,
    ratio_low_slider_state: nih_widgets::param_slider::State,
    attack_low_slider_state: nih_widgets::param_slider::State,
    release_low_slider_state: nih_widgets::param_slider::State,
    makeup_low_slider_state: nih_widgets::param_slider::State,

    // Mid band sliders
    threshold_mid_slider_state: nih_widgets::param_slider::State,
    ratio_mid_slider_state: nih_widgets::param_slider::State,
    attack_mid_slider_state: nih_widgets::param_slider::State,
    release_mid_slider_state: nih_widgets::param_slider::State,
    makeup_mid_slider_state: nih_widgets::param_slider::State,

    // High band sliders
    threshold_high_slider_state: nih_widgets::param_slider::State,
    ratio_high_slider_state: nih_widgets::param_slider::State,
    attack_high_slider_state: nih_widgets::param_slider::State,
    release_high_slider_state: nih_widgets::param_slider::State,
    makeup_high_slider_state: nih_widgets::param_slider::State,

    // Crossover sliders
    xover_lo_mid_state: nih_widgets::param_slider::State,
    xover_mid_hi_state: nih_widgets::param_slider::State,

    peak_meter_state: nih_widgets::peak_meter::State,
    scrollable_state: scrollable::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    /// Update a parameter's value.
    ParamUpdate(nih_widgets::ParamMessage),
}

impl IcedEditor for MultibandCompressorEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = (Arc<MultibandCompressorParams>, Arc<AtomicF32>);

    fn new(
        (params, peak_meter): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = MultibandCompressorEditor {
            params,
            context,

            peak_meter,

            // Low band
            threshold_low_slider_state: Default::default(),
            ratio_low_slider_state: Default::default(),
            attack_low_slider_state: Default::default(),
            release_low_slider_state: Default::default(),
            makeup_low_slider_state: Default::default(),

            // Mid band
            threshold_mid_slider_state: Default::default(),
            ratio_mid_slider_state: Default::default(),
            attack_mid_slider_state: Default::default(),
            release_mid_slider_state: Default::default(),
            makeup_mid_slider_state: Default::default(),

            // High band
            threshold_high_slider_state: Default::default(),
            ratio_high_slider_state: Default::default(),
            attack_high_slider_state: Default::default(),
            release_high_slider_state: Default::default(),
            makeup_high_slider_state: Default::default(),

            // Crossovers
            xover_lo_mid_state: Default::default(),
            xover_mid_hi_state: Default::default(),

            peak_meter_state: Default::default(),
            scrollable_state: Default::default(),
        };

        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        message: Self::Message,
    ) -> Command<Self::Message> {
        match message {
            Message::ParamUpdate(message) => self.handle_param_message(message),
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Scrollable::new(&mut self.scrollable_state)
            .push(
                Column::new()
                    .align_items(Alignment::Center)
                    .padding(20)
                    .spacing(10)
                    .push(
                        Text::new("Multiband Compressor")
                            .font(assets::NOTO_SANS_LIGHT)
                            .size(24)
                            .height(30.into())
                            .width(Length::Fill)
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .vertical_alignment(alignment::Vertical::Bottom),
                    )
                    .push(Space::with_height(10.into()))
                    .push(
                        Row::new()
                            .spacing(20)
                            .width(Length::Fill)
                            .push(
                                Column::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .width(Length::Fill)
                                    .push(
                                        Text::new("Low Band")
                                            .font(assets::NOTO_SANS_LIGHT)
                                            .size(18)
                                            .width(Length::Fill)
                                            .horizontal_alignment(alignment::Horizontal::Center),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.threshold_low_slider_state,
                                            &self.params.threshold_low,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.ratio_low_slider_state,
                                            &self.params.ratio_low,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.attack_low_slider_state,
                                            &self.params.attack_low,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.release_low_slider_state,
                                            &self.params.release_low,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.makeup_low_slider_state,
                                            &self.params.makeup_low,
                                        )
                                        .map(Message::ParamUpdate),
                                    ),
                            )
                            .push(
                                Column::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .width(Length::Fill)
                                    .push(
                                        Text::new("Mid Band")
                                            .font(assets::NOTO_SANS_LIGHT)
                                            .size(18)
                                            .width(Length::Fill)
                                            .horizontal_alignment(alignment::Horizontal::Center),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.threshold_mid_slider_state,
                                            &self.params.threshold_mid,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.ratio_mid_slider_state,
                                            &self.params.ratio_mid,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.attack_mid_slider_state,
                                            &self.params.attack_mid,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.release_mid_slider_state,
                                            &self.params.release_mid,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.makeup_mid_slider_state,
                                            &self.params.makeup_mid,
                                        )
                                        .map(Message::ParamUpdate),
                                    ),
                            )
                            .push(
                                Column::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .width(Length::Fill)
                                    .push(
                                        Text::new("High Band")
                                            .font(assets::NOTO_SANS_LIGHT)
                                            .size(18)
                                            .width(Length::Fill)
                                            .horizontal_alignment(alignment::Horizontal::Center),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.threshold_high_slider_state,
                                            &self.params.threshold_high,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.ratio_high_slider_state,
                                            &self.params.ratio_high,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.attack_high_slider_state,
                                            &self.params.attack_high,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.release_high_slider_state,
                                            &self.params.release_high,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.makeup_high_slider_state,
                                            &self.params.makeup_high,
                                        )
                                        .map(Message::ParamUpdate),
                                    ),
                            ),
                    )
                    .push(Space::with_height(10.into()))
                    .push(
                        Row::new()
                            .spacing(20)
                            .width(Length::Fill)
                            .align_items(Alignment::Start)
                            .push(
                                Column::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .width(Length::Fill)
                                    .push(
                                        Text::new("Crossovers")
                                            .font(assets::NOTO_SANS_LIGHT)
                                            .size(18)
                                            .width(Length::Fill)
                                            .horizontal_alignment(alignment::Horizontal::Center),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.xover_lo_mid_state,
                                            &self.params.xover_lo_mid,
                                        )
                                        .map(Message::ParamUpdate),
                                    )
                                    .push(
                                        nih_widgets::ParamSlider::new(
                                            &mut self.xover_mid_hi_state,
                                            &self.params.xover_mid_hi,
                                        )
                                        .map(Message::ParamUpdate),
                                    ),
                            )
                            .push(
                                Column::new()
                                    .align_items(Alignment::Center)
                                    .spacing(10)
                                    .width(Length::Shrink)
                                    .push(
                                        Text::new("Peak Meter")
                                            .font(assets::NOTO_SANS_LIGHT)
                                            .size(18)
                                            .horizontal_alignment(alignment::Horizontal::Center),
                                    )
                                    .push(
                                        nih_widgets::PeakMeter::new(
                                            &mut self.peak_meter_state,
                                            util::gain_to_db(
                                                self.peak_meter
                                                    .load(std::sync::atomic::Ordering::Relaxed),
                                            ),
                                        )
                                        .hold_time(Duration::from_millis(600)),
                                    ),
                            ),
                    )
                    .push(Space::with_height(20.into())),
            )
            .into()
    }

    fn background_color(&self) -> nih_plug_iced::Color {
        nih_plug_iced::Color {
            r: 0.98,
            g: 0.98,
            b: 0.98,
            a: 1.0,
        }
    }
}
