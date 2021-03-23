use crate::frame_view::{Frame, FRAME_HEIGHT};
use crate::profile_data::{FrameDataObject, ProfileData};
use gtk4::gio::ListStore;
use gtk4::glib::types::Type;
use gtk4::prelude::Cast;
use gtk4::{
    ListView, NoSelection, OrientableExt, Orientation, ScrolledWindow, SignalListItemFactory,
    NONE_SELECTION_MODEL, NONE_WIDGET,
};

pub struct FrameTimeline {
    widget: ScrolledWindow,
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

        let widget = ScrolledWindow::new();
        widget.set_min_content_height(FRAME_HEIGHT + 10);
        widget.set_child(Some(&list_view));

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

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}
