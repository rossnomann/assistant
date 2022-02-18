use std::collections::HashMap;
use tokio_postgres::Row;

const MAX_LIST_ITEM_LEN: usize = 4096;

pub struct NoteInfoList {
    items: Vec<NoteInfo>,
    current_index: usize,
}

impl NoteInfoList {
    fn new(items: Vec<NoteInfo>) -> Self {
        Self {
            items,
            current_index: 0,
        }
    }
}

impl Iterator for NoteInfoList {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let total_items = self.items.len();
        if self.current_index >= total_items {
            None
        } else {
            let mut size = 0;
            let mut result = Vec::new();
            for idx in self.current_index..total_items {
                let item = self.items[idx].as_string();
                let item_len = item.len();
                if size + item_len > MAX_LIST_ITEM_LEN {
                    break;
                }
                result.push(item);
                size += item_len;
                self.current_index += 1;
            }
            Some(result.join("\n"))
        }
    }
}

impl From<Vec<Row>> for NoteInfoList {
    fn from(rows: Vec<Row>) -> Self {
        let items = rows.into_iter().map(NoteInfo::from).collect();
        Self::new(items)
    }
}

#[derive(Debug)]
pub struct NoteInfo {
    id: i32,
    keywords: Vec<String>,
}

impl NoteInfo {
    fn as_string(&self) -> String {
        let mut result = format!(r#"`{}` \- {}"#, self.id, self.keywords.join(" "));
        if result.len() > MAX_LIST_ITEM_LEN {
            result = result.chars().take(MAX_LIST_ITEM_LEN - 3).collect();
            result.push_str("...");
        }
        result
    }
}

impl From<Row> for NoteInfo {
    fn from(row: Row) -> Self {
        let indexes: HashMap<&str, usize> = row
            .columns()
            .iter()
            .enumerate()
            .map(|(idx, column)| (column.name(), idx))
            .collect();
        Self {
            id: row.get(indexes["id"]),
            keywords: row.get(indexes["keywords"]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_note_info<T, I>(id: i32, keywords: T) -> NoteInfo
    where
        T: IntoIterator<Item = I>,
        I: Into<String>,
    {
        NoteInfo {
            id,
            keywords: keywords.into_iter().map(Into::into).collect(),
        }
    }

    #[test]
    fn note_info_list() {
        let list = NoteInfoList::new(vec![
            create_note_info(1, vec!["k1", "k2"]),
            create_note_info(2, vec!["k3", "k4"]),
            create_note_info(3, vec!["k".repeat(MAX_LIST_ITEM_LEN * 2)]),
        ]);
        let formatted_list: Vec<String> = list.collect();
        const PREFIX_LEN: usize = 7;
        assert_eq!(
            formatted_list,
            &[
                String::from("`1` \\- k1 k2\n`2` \\- k3 k4"),
                format!(r#"`3` \- {}..."#, "k".repeat(MAX_LIST_ITEM_LEN - 3 - PREFIX_LEN))
            ]
        )
    }
}
