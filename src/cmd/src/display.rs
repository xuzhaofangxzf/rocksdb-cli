use anyhow::Result;
use comfy_table::{Cell, Color, Table};
use rocksdb::DB;
const BATH_ROWS: usize = 100;

pub fn print_key_value(key: &[u8], value: &[u8]) {
    let mut table = Table::new();
    table.set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth);
    table.set_header(vec![
        Cell::new("Key")
            .add_attribute(comfy_table::Attribute::Bold)
            .set_alignment(comfy_table::CellAlignment::Center)
            .fg(Color::Green),
        Cell::new("Value")
            .add_attribute(comfy_table::Attribute::Bold)
            .set_alignment(comfy_table::CellAlignment::Center)
            .fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new(String::from_utf8_lossy(key)),
        Cell::new(
            match unescaper::unescape(String::from_utf8_lossy(value).as_ref()) {
                Ok(s_value) => s_value,
                Err(_) => String::from_utf8_lossy(value).to_string(),
            },
        ),
    ]);
    println!("{table}");
}

pub fn print_key_value_list<T: Iterator<Item = (Vec<u8>, Vec<u8>)>>(entries: T) {
    let mut table = Table::new();
    table.set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth);
    table.set_header(vec![
        Cell::new("Key")
            .add_attribute(comfy_table::Attribute::Bold)
            .set_alignment(comfy_table::CellAlignment::Center)
            .fg(Color::Green),
        Cell::new("Value")
            .add_attribute(comfy_table::Attribute::Bold)
            .set_alignment(comfy_table::CellAlignment::Center)
            .fg(Color::Green),
    ]);
    table.set_row_capacity(BATH_ROWS);
    let mut row_count = 0;
    for (key, value) in entries {
        let key_str = String::from_utf8_lossy(&key).into_owned();
        let value_str = match std::str::from_utf8(&value) {
            Ok(s) => match unescaper::unescape(s) {
                Ok(es) => es,
                Err(_) => s.to_string(),
            },
            Err(_) => format!("[BINARY] {}", hex::encode(value)),
        };
        table.add_row(vec![key_str, value_str]);
        row_count += 1;
        if row_count % BATH_ROWS == 0 {
            println!("{table}");
            table.clear_rows();
        }
    }
    if !table.is_empty() {
        println!("{table}");
    }
}

pub fn print_column_families(cfs: &[String], current: &str) {
    let mut table = Table::new();
    table.set_header(vec!["Column Family", "Status"]);

    for cf in cfs {
        if cf == current {
            table.add_row(vec![
                Cell::new(cf),
                Cell::new("Active")
                    .add_attribute(comfy_table::Attribute::Bold)
                    .fg(Color::Green),
            ]);
        } else {
            table.add_row(vec![Cell::new(cf), Cell::new("Avaliable")]);
        }
    }
    println!("{table}");
}

pub fn print_database_info(db: &DB, path: &str, current_cf: &str) -> Result<()> {
    let mut table = Table::new();
    table.set_header(vec!["Property", "Value"]);

    table.add_row(vec!["Path", path]);
    table.add_row(vec!["Current Column Family", current_cf]);

    if let Some(create_time) = db.property_value("rocksdb.creation-time")? {
        table.add_row(vec!["Creation Time", &create_time]);
    }

    if let Some(version) = db.property_value("rocksdb.version")? {
        table.add_row(vec!["Version", &version]);
    }

    if let Some(num_files) = db.property_value("rocksdb.num-files-at-level0")? {
        table.add_row(vec!["L0 Files", &num_files]);
    }

    if let Some(size) = db.property_value("rocksdb.total-sst-files-size")? {
        table.add_row(vec!["Total SST Size", &format!("{} bytes", size)]);
    }

    println!("{table}");
    Ok(())
}
