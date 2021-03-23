use crate::frame_view::{Frame, FrameThreshold, FRAME_HEIGHT};
use crate::profile_data::{FrameDataObject, ProfileData};
use gtk4::gio::ListStore;
use gtk4::glib::types::Type;
use gtk4::prelude::Cast;
use gtk4::{
    Align, ListView, NoSelection, OrientableExt, Orientation, Overlay, ScrolledWindow,
    SignalListItemFactory, WidgetExt, NONE_SELECTION_MODEL, NONE_WIDGET,
};
use std::time::Duration;

pub struct FrameTimeline {
    widget: Overlay,
    list_view: ListView,
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

        let widget = Overlay::new();
        widget.set_child(Some(&scrolled_window));
        widget.add_overlay(frame_threshold.widget());

        Self { widget, list_view }
    }

    pub fn load_profile(&self, profile: &ProfileData) {
        let model = ListStore::new(Type::OBJECT);
        for frame_data in profile.frames.iter() {
            let obj = FrameDataObject::new(frame_data.clone());
            model.append(&obj);
        }

        let selection_model = NoSelection::new(Some(&model));

        self.list_view.set_model(Some(&selection_model));
    }

    pub fn widget(&self) -> &Overlay {
        &self.widget
    }
}
