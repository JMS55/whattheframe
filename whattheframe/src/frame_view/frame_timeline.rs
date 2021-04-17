use crate::frame_view::{Frame, FrameThreshold, FRAME_HEIGHT};
use crate::task_object::TaskObject;
use gtk4::gio::ListStore;
use gtk4::glib::types::Type;
use gtk4::prelude::Cast;
use gtk4::{
    Align, CheckButton, CheckButtonExt, CustomFilter, FilterListModel, ListView, OrientableExt,
    Orientation, Overlay, ScrolledWindow, SelectionModelExt, SignalListItemFactory,
    SingleSelection, WidgetExt, NONE_FILTER, NONE_SELECTION_MODEL, NONE_WIDGET,
};
use std::time::Duration;

pub struct FrameTimeline {
    widget: Overlay,
    list_view: ListView,
    threshold_toggle: CheckButton,
}

impl FrameTimeline {
    pub fn new() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            list_item.set_child(Some(&Frame::new()));
        });
        factory.connect_bind(|_, list_item| {
            let frame = list_item.get_child().unwrap().downcast::<Frame>().unwrap();
            let frame_data = list_item
                .get_item()
                .unwrap()
                .downcast::<TaskObject>()
                .unwrap();
            frame.set_data(frame_data);
        });
        factory.connect_teardown(|_, list_item| {
            list_item.set_child(NONE_WIDGET);
        });

        let list_view = ListView::new(NONE_SELECTION_MODEL, Some(&factory));
        list_view.set_orientation(Orientation::Horizontal);
        list_view.add_css_class("frame-timeline");

        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_min_content_height(FRAME_HEIGHT + 10);
        scrolled_window.set_child(Some(&list_view));

        let frame_threshold = FrameThreshold::new();
        frame_threshold.widget().set_valign(Align::Start);
        // TODO: Clean margin calculation up
        let margin = (FRAME_HEIGHT - 21)
            - ((((Duration::from_nanos(16666671).as_secs_f64() * 1000.0) / 24.0)
                * (FRAME_HEIGHT - 21) as f64)
                .round() as i32);
        frame_threshold.widget().set_margin_top(margin);

        let threshold_toggle = CheckButton::new();
        threshold_toggle.set_halign(Align::Start);
        threshold_toggle.set_valign(Align::Start);

        let widget = Overlay::new();
        widget.set_child(Some(&scrolled_window));
        widget.add_overlay(frame_threshold.widget());
        widget.add_overlay(&threshold_toggle);

        Self {
            widget,
            list_view,
            threshold_toggle,
        }
    }

    pub fn load_frames<F>(
        &self,
        frames: &[TaskObject],
        above_threshold_count: usize,
        on_frame_selection_change: F,
    ) where
        F: Fn(Option<TaskObject>) + 'static,
    {
        let model = ListStore::new(Type::OBJECT);
        for frame in frames {
            model.append(frame);
        }
        let model = FilterListModel::new(Some(&model), NONE_FILTER);

        self.threshold_toggle.set_label(Some(&format!(
            "Filter Threshhold ({})",
            above_threshold_count
        )));
        self.threshold_toggle.connect_toggled({
            let model = model.clone();
            move |threshold_toggle| {
                if threshold_toggle.get_active() {
                    model.set_filter(Some(&CustomFilter::new(|item| {
                        item.downcast_ref::<TaskObject>().unwrap().get().duration
                            > Duration::from_nanos(16666670)
                    })));
                } else {
                    model.set_filter(NONE_FILTER);
                }
            }
        });

        let model = SingleSelection::new(Some(&model));
        model.set_can_unselect(true);
        model.connect_selection_changed(move |model, _, _| {
            let task_data = model
                .get_selected_item()
                .map(|d| d.downcast::<TaskObject>().unwrap());
            (on_frame_selection_change)(task_data);
        });

        self.list_view.set_model(Some(&model));
    }

    pub fn widget(&self) -> &Overlay {
        &self.widget
    }
}
