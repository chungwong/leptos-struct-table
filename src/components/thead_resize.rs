use leptos::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Copy)]
pub struct ColumnWidths<Column: Eq + Hash + Copy + Send + Sync + 'static> {
    pub widths: RwSignal<HashMap<Column, f64>>,
    pub resizing: RwSignal<Option<Column>>,
}

impl<Column: Eq + Hash + Copy + Send + Sync + 'static> ColumnWidths<Column> {
    pub fn new() -> Self {
        Self {
            widths: RwSignal::new(HashMap::new()),
            resizing: RwSignal::new(None),
        }
    }

    pub fn get_width(&self, column: Column) -> Option<f64> {
        self.widths.with(|w| w.get(&column).copied())
    }

    pub fn set_width(&self, column: Column, width: f64) {
        self.widths.update(|w| {
            w.insert(column, width);
        });
    }

    pub fn style_for(&self, column: Column) -> Signal<String> {
        let widths = self.widths;
        Signal::derive(move || {
            widths.with(|w| {
                w.get(&column)
                    .map(|width| {
                        format!("width:{width}px;min-width:{width}px;max-width:{width}px;")
                    })
                    .unwrap_or_default()
            })
        })
    }
}

impl<Column: Eq + Hash + Copy + Send + Sync + 'static> Default for ColumnWidths<Column> {
    fn default() -> Self {
        Self::new()
    }
}
