use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor, GuiContext};
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;
use std::time::Duration;

use crate::SimpleCompressorParams;

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<IcedState> {
    IcedState::from_size(400, 500)
}

pub(crate) fn create(
    params: Arc<SimpleCompressorParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<SimpleCompressorEditor>(editor_state, (params, peak_meter))
}

struct SimpleCompressorEditor {
    params: Arc<SimpleCompressorParams>,
    context: Arc<dyn GuiContext>,

    peak_meter: Arc<AtomicF32>,

    threshold_slider_state: nih_widgets::param_slider::State,
    ratio_slider_state: nih_widgets::param_slider::State,
    attack_slider_state: nih_widgets::param_slider::State,
    release_slider_state: nih_widgets::param_slider::State,
    makeup_slider_state: nih_widgets::param_slider::State,
    peak_meter_state: nih_widgets::peak_meter::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    /// Update a parameter's value.
    ParamUpdate(nih_widgets::ParamMessage),
}

impl IcedEditor for SimpleCompressorEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = (Arc<SimpleCompressorParams>, Arc<AtomicF32>);

    fn new(
        (params, peak_meter): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = SimpleCompressorEditor {
            params,
            context,

            peak_meter,

            threshold_slider_state: Default::default(),
            ratio_slider_state: Default::default(),
            attack_slider_state: Default::default(),
            release_slider_state: Default::default(),
            makeup_slider_state: Default::default(),
            peak_meter_state: Default::default(),
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
        Column::new()
            .align_items(Alignment::Center)
            .padding(20)
            .spacing(10)
            .push(
                Text::new("Simple Compressor")
                    .font(assets::NOTO_SANS_LIGHT)
                    .size(24)
                    .height(30.into())
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Bottom),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.threshold_slider_state, &self.params.threshold)
                    .map(Message::ParamUpdate),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.ratio_slider_state, &self.params.ratio)
                    .map(Message::ParamUpdate),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.attack_slider_state, &self.params.attack)
                    .map(Message::ParamUpdate),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.release_slider_state, &self.params.release)
                    .map(Message::ParamUpdate),
            )
            .push(
                nih_widgets::ParamSlider::new(&mut self.makeup_slider_state, &self.params.makeup)
                    .map(Message::ParamUpdate),
            )
            .push(Space::with_height(20.into()))
            .push(
                nih_widgets::PeakMeter::new(
                    &mut self.peak_meter_state,
                    util::gain_to_db(self.peak_meter.load(std::sync::atomic::Ordering::Relaxed)),
                )
                .hold_time(Duration::from_millis(600)),
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