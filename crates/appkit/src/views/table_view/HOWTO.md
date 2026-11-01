# TableView — How to Use

## Files
- `mod.rs` — `NSTableView`, `NSTableColumn`, `NSTableViewDataSource` trait, `NSTableViewDelegate` trait
- `delegate.rs` — `NSTableViewRenderDelegate` handles drawing rows/columns

## Quick Start
```rust
use rust_appkit::views::table_view::{NSTableView, NSTableColumn, NSTableViewDataSource};

struct MyData { rows: Vec<(String, String)> }

impl NSTableViewDataSource for MyData {
    fn number_of_rows(&self) -> usize { self.rows.len() }
    fn value_for(&self, row: usize, column: &str) -> String {
        match column {
            "name" => self.rows[row].0.clone(),
            "type" => self.rows[row].1.clone(),
            _ => String::new(),
        }
    }
}

let mut table = NSTableView::new(frame);
table.add_column(NSTableColumn::new("name", "Name", 200.0));
table.add_column(NSTableColumn::new("type", "Type", 100.0));
table.set_data_source(MyData { rows: vec![
    ("Cube".into(), "Mesh".into()),
    ("Light".into(), "Point".into()),
]});

// Select a row
table.select_row(Some(0));
```

## Key APIs
- `NSTableView::new(frame)` — create the table
- `table.add_column(col)` — define a column
- `table.set_data_source(ds)` — provide data + install render delegate
- `table.select_row(Some(i))` — programmatic selection
- `table.reload_data()` — refresh from data source
- `NSTableColumn::new(id, title, width)` — column definition
- `.with_min_width(w)` / `.with_max_width(w)` — column constraints

## Rules
- Setting data source auto-installs the render delegate
- Row height comes from theme (`SIZE_CONTROL_HEIGHT`, default 24px)
- Column identifiers are used as keys in `value_for(row, column)`
- Call `reload_data()` after external data changes
