use crate::frame_view::{Frame, FrameThreshold, FRAME_HEIGHT};
use crate::profile_data::{FrameDataObject, ProfileData};
use gtk4::gio::ListStore;
use gtk4::glib::types::Type;
use gtk4::prelude::Cast;
use gtk4::{
    Align, Button, CheckButton, CheckButtonExt, CustomFilter, FilterListModel, ListView,
    NoSelection, OrientableExt, Orientation, Overlay, ScrolledWindow, SignalListItemFactory, Stack,
    WidgetExt, NONE_FILTER, NONE_SELECTION_MODEL, NONE_WIDGET,
};
use std::time::Duration;

pub struct FrameTimeline {
    widget: Stack,
    placeholder_widget: Button,
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
                .downcast::<FrameDataObject>()
                .unwrap();
            frame.set_data(Some(frame_data));
        });
        factory.connect_unbind(|_, list_item| {
            let frame = list_item.get_child().unwrap().downcast::<Frame>().unwrap();
            frame.set_data(None);
        });
        factory.connect_teardown(|_, list_item| {
            list_item.set_child(NONE_WIDGET);
        });

        let list_view = ListView::new(NONE_SELECTION_MODEL, Some(&factory));
        list_view.set_orientation(Orientation::Horizontal);

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
        threshold_toggle.set_valign(Align::Start);

        let content = Overlay::new();
        content.set_child(Some(&scrolled_window));
        content.add_overlay(frame_threshold.widget());
        content.add_overlay(&threshold_toggle);

        let placeholder_widget = Button::with_label("Click To Load A Profile");
        placeholder_widget.add_css_class("title-1");

        let widget = Stack::new();
        widget.add_named(&placeholder_widget, Some("placeholder"));
        widget.add_named(&content, Some("content"));

        Self {
            widget,
            placeholder_widget,
            list_view,
            threshold_toggle,
        }
    }

    pub fn load_profile(&self, profile: &ProfileData) {
        let mut above_threshold_count = 0;

        let model = ListStore::new(Type::OBJECT);
        for frame_data in profile.frames.iter() {
            let obj = FrameDataObject::new(frame_data.clone());
            model.append(&obj);

            if frame_data.duration > Duration::from_nanos(16666670) {
                above_threshold_count += 1;
            }
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
                        item.downcast_ref::<FrameDataObject>()
                            .unwrap()
                            .get()
                            .duration
                            > Duration::from_nanos(16666670)
                    })));
                } else {
                    model.set_filter(NONE_FILTER);
                }
            }
        });

        let model = NoSelection::new(Some(&model));
        self.list_view.set_model(Some(&model));

        self.widget.set_visible_child_name("content");
    }

    pub fn widget(&self) -> &Stack {
        &self.widget
    }

    pub fn placeholder_widget(&self) -> &Button {
        &self.placeholder_widget
    }
}
