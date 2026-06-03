use crate::wrapper_render_fn;
use crate::{ColumnSort, ColumnWidths, DragStateRwSignal, HeadDragHandler, TableHeadEvent};
use leptos::prelude::*;
use std::hash::Hash;

wrapper_render_fn!(
    /// thead
    DefaultTableHeadRenderer,
    thead,
);

wrapper_render_fn!(
    /// thead row
    DefaultTableHeadRowRenderer,
    tr,
);

/// The default table header renderer with optional column resizing.
///
/// When [`ColumnWidths`] is provided in context (via `TableContent`'s `column_widths` prop),
/// a resize handle appears on the right edge of each header cell.
#[component]
pub fn DefaultTableHeaderCellRenderer<F, Column>(
    #[prop(into)] class: Signal<String>,
    #[prop(into)] inner_class: String,
    index: Column,
    #[prop(into)] sort_priority: Signal<Option<usize>>,
    #[prop(into)] sort_direction: Signal<ColumnSort>,
    on_click: F,
    drag_state: DragStateRwSignal<Column>,
    drag_handler: HeadDragHandler<Column>,
    columns: RwSignal<Vec<Column>>,
    children: Children,
) -> impl IntoView
where
    F: Fn(TableHeadEvent<Column>) + 'static,
    Column: Eq + Copy + Hash + Send + Sync + std::fmt::Debug + 'static,
{
    let sort_style = default_th_sorting_style(sort_priority, sort_direction);
    let drag_classes = drag_handler.0.get_drag_classes(drag_state, index, columns);
    let col_widths = use_context::<ColumnWidths<Column>>();
    let resizable = col_widths.is_some();
    let th_ref = NodeRef::<leptos::html::Th>::new();

    let th_style = Signal::derive(move || {
        let mut s = sort_style.get();
        if let Some(ref cw) = col_widths {
            cw.widths.with(|w| {
                if let Some(width) = w.get(&index) {
                    s.push_str(&format!(
                        "width:{width}px;min-width:{width}px;max-width:{width}px;"
                    ));
                }
            });
            s.push_str("position:relative;overflow:visible;");
        }
        s
    });

    let resize_handle = resizable.then(|| {
        let cw = col_widths.clone().unwrap();
        let widths = cw.widths;
        let resizing = cw.resizing;
        let drag_start_x = RwSignal::new(0.0_f64);
        let drag_start_width = RwSignal::new(0.0_f64);

        let do_move = move |client_x: f64| {
            if resizing.get_untracked() != Some(index) {
                return;
            }
            let dx = client_x - drag_start_x.get_untracked();
            let new_width = (drag_start_width.get_untracked() + dx).max(40.0);
            widths.update(|w| {
                w.insert(index, new_width);
            });
        };

        let end_resize = move || {
            if resizing.get_untracked() == Some(index) {
                resizing.set(None);
            }
        };

        let _ = leptos_use::use_event_listener(leptos::prelude::document(), leptos::ev::mousemove, move |ev| {
            do_move(ev.client_x() as f64);
        });
        let _ = leptos_use::use_event_listener(leptos::prelude::document(), leptos::ev::touchmove, move |ev| {
            if let Some(touch) = ev.touches().get(0) {
                do_move(touch.client_x() as f64);
            }
        });
        let _ = leptos_use::use_event_listener(leptos::prelude::document(), leptos::ev::mouseup, move |_| {
            end_resize();
        });
        let _ = leptos_use::use_event_listener(leptos::prelude::document(), leptos::ev::touchend, move |_| {
            end_resize();
        });

        let start_resize = move |client_x: f64| {
            drag_start_x.set(client_x);
            let current_width = widths
                .with_untracked(|w| w.get(&index).copied())
                .unwrap_or_else(|| {
                    th_ref
                        .get()
                        .map(|th| {
                            let el: &web_sys::Element = &th;
                            el.get_bounding_client_rect().width()
                        })
                        .unwrap_or(100.0)
                });
            drag_start_width.set(current_width);
            resizing.set(Some(index));
        };

        let on_mousedown = move |ev: web_sys::MouseEvent| {
            ev.prevent_default();
            ev.stop_propagation();
            start_resize(ev.client_x() as f64);
        };

        let on_touchstart = move |ev: web_sys::TouchEvent| {
            ev.stop_propagation();
            if let Some(touch) = ev.touches().get(0) {
                start_resize(touch.client_x() as f64);
            }
        };

        view! {
            <div
                style="position:absolute;right:-4px;top:0;bottom:0;width:12px;cursor:col-resize;z-index:1;display:flex;align-items:center;justify-content:center;touch-action:none;"
                on:mousedown=on_mousedown
                on:touchstart=on_touchstart
            >
                <div style="width:2px;height:60%;background:var(--border-strong);border-radius:1px;" />
            </div>
        }
    });

    view! {
        <th
            node_ref=th_ref
            class=move || format!("{} {}", class.get(), drag_classes.get())
            style=th_style
            draggable="true"
            on:click=move |mouse_event| on_click(TableHeadEvent {
                index,
                mouse_event,
            })
            on:drop={
                let drag_handler = drag_handler.clone();
                move |evt| {
                    drag_handler.0.received_drop(drag_state, columns, index, evt);
                }
            }
            on:dragover={
                let drag_handler = drag_handler.clone();
                move |evt| {
                    drag_handler.0.dragging_over(drag_state, index, evt);
                }
            }
            on:dragleave={
                let drag_handler = drag_handler.clone();
                move |evt| {
                    drag_handler.0.drag_leave(drag_state, index, evt);
                }
            }
            on:dragstart={
                let drag_handler = drag_handler.clone();
                move |evt| {
                    drag_handler.0.drag_start(drag_state, index, evt);
                }
            }
            on:dragend=move |evt| {
                drag_handler.0.drag_end(drag_state, columns, index, evt);
            }
        >
            <span class=inner_class>{children()}</span>
            {resize_handle}
        </th>
    }
}

/// You can use this function to implement your own custom table header cell renderer.
///
/// See the implementation of [`DefaultTableHeaderCellRenderer`].
pub fn default_th_sorting_style(
    sort_priority: Signal<Option<usize>>,
    sort_direction: Signal<ColumnSort>,
) -> Signal<String> {
    Signal::derive(move || {
        let sort = match sort_direction.get() {
            ColumnSort::Ascending => "--sort-icon: '▲';",
            ColumnSort::Descending => "--sort-icon: '▼';",
            ColumnSort::None => "--sort-icon: '';",
        };

        let priority = match sort_priority.get() {
            Some(priority) => format!("--sort-priority: '{}';", priority + 1),
            None => "--sort-priority: '';".to_string(),
        };

        format!("{} {}", sort, &priority)
    })
}
