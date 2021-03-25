use crate::profile_data::TaskObject;
use gtk4::gio::{ListModel, ListStore};
use gtk4::glib::Type;
use gtk4::prelude::Cast;
use gtk4::{
    Label, ListView, NoSelection, ScrolledWindow, SignalListItemFactory, TreeExpander,
    TreeListModel, TreeListRow, NONE_SELECTION_MODEL, NONE_WIDGET,
};
use std::cmp::Reverse;

pub struct TaskTree {
    widget: ScrolledWindow,
    list_view: ListView,
}

impl TaskTree {
    pub fn new() -> Self {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(|_, list_item| {
            let row_expander = TreeExpander::new();
            row_expander.set_child(Some(&Label::new(None)));
            list_item.set_child(Some(&row_expander));
        });
        factory.connect_bind(|_, list_item| {
            let row = list_item
                .get_item()
                .unwrap()
                .downcast::<TreeListRow>()
                .unwrap();
            let row_expander = list_item
                .get_child()
                .unwrap()
                .downcast::<TreeExpander>()
                .unwrap();
            row_expander.set_list_row(Some(&row));

            let label = row_expander
                .get_child()
                .unwrap()
                .downcast::<Label>()
                .unwrap();
            let task = row.get_item().unwrap().downcast::<TaskObject>().unwrap();

            let task_duration_ms = task.get().duration.as_secs_f64() * 1000.0;
            label.set_label(&format!("{} ({:.2}ms)", task.get().name, task_duration_ms));
        });
        factory.connect_unbind(|_, list_item| {
            let row_expander = list_item
                .get_child()
                .unwrap()
                .downcast::<TreeExpander>()
                .unwrap();
            let label = row_expander
                .get_child()
                .unwrap()
                .downcast::<Label>()
                .unwrap();
            row_expander.set_list_row(None);
            label.set_label("");
        });
        factory.connect_teardown(|_, list_item| {
            list_item.set_child(NONE_WIDGET);
        });

        let list_view = ListView::new(NONE_SELECTION_MODEL, Some(&factory));

        let widget = ScrolledWindow::new();
        widget.set_child(Some(&list_view));

        Self { widget, list_view }
    }

    pub fn set_frame(&self, frame: Option<TaskObject>) {
        match frame {
            Some(frame) => {
                let model = ListStore::new(Type::OBJECT);
                model.append(&frame);
                let model = TreeListModel::new(&model, false, false, |item| {
                    let subtasks = &item.downcast_ref::<TaskObject>().unwrap().get().subtasks;
                    if subtasks.is_empty() {
                        return None;
                    }
                    let mut subtasks = subtasks.clone();
                    subtasks.sort_by_key(|task| Reverse(task.duration));

                    let model = ListStore::new(Type::OBJECT);
                    for subtask in subtasks.iter() {
                        let subtask = TaskObject::new(subtask.clone());
                        model.append(&subtask);
                    }
                    Some(model.upcast::<ListModel>())
                });
                let model = NoSelection::new(Some(&model));
                self.list_view.set_model(Some(&model));
            }
            None => self.list_view.set_model(NONE_SELECTION_MODEL),
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}
