use ansi_term::ANSIStrings;

use crate::{LinePart, ARROW_SEPARATOR};
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

fn get_current_title_len(current_title: &[LinePart]) -> usize {
    current_title.iter().map(|p| p.len).sum()
}

// move elements from before_active and after_active into tabs_to_render while they fit in cols
// adds collapsed_tabs to the left and right if there's left over tabs that don't fit
fn populate_tabs_in_tab_line(
    tabs_before_active: &mut Vec<LinePart>,
    tabs_after_active: &mut Vec<LinePart>,
    tabs_to_render: &mut Vec<LinePart>,
    cols: usize,
    palette: Palette,
    capabilities: PluginCapabilities,
) {
    let mut middle_size = get_current_title_len(tabs_to_render);

    let mut total_left = 0;
    let mut total_right = 0;
    loop {
        let left_count = tabs_before_active.len();
        let right_count = tabs_after_active.len();
        let collapsed_left = left_more_message(left_count, palette, tab_separator(capabilities));
        let collapsed_right = right_more_message(right_count, palette, tab_separator(capabilities));

        let total_size = collapsed_left.len + middle_size + collapsed_right.len;

        if total_size > cols {
            // break and dont add collapsed tabs to tabs_to_render, they will not fit
            break;
        }

        let left = if let Some(tab) = tabs_before_active.last() {
            tab.len
        } else {
            usize::MAX
        };

        let right = if let Some(tab) = tabs_after_active.first() {
            tab.len
        } else {
            usize::MAX
        };

        // total size is shortened if the next tab to be added is the last one, as that will remove the collapsed tab
        let size_by_adding_left =
            left.saturating_add(total_size)
                .saturating_sub(if left_count == 1 {
                    collapsed_left.len
                } else {
                    0
                });
        let size_by_adding_right =
            right
                .saturating_add(total_size)
                .saturating_sub(if right_count == 1 {
                    collapsed_right.len
                } else {
                    0
                });

        let left_fits = size_by_adding_left <= cols;
        let right_fits = size_by_adding_right <= cols;
        // active tab is kept in the middle by adding to the side that
        // has less width, or if the tab on the other side doesn' fit
        if (total_left <= total_right || !right_fits) && left_fits {
            // add left tab
            let tab = tabs_before_active.pop().unwrap();
            middle_size += tab.len;
            total_left += tab.len;
            tabs_to_render.insert(0, tab);
        } else if right_fits {
            // add right tab
            let tab = tabs_after_active.remove(0);
            middle_size += tab.len;
            total_right += tab.len;
            tabs_to_render.push(tab);
        } else {
            // there's either no space to add more tabs or no more tabs to add, so we're done
            tabs_to_render.insert(0, collapsed_left);
            tabs_to_render.push(collapsed_right);
            break;
        }
    }
}

fn left_more_message(tab_count_to_the_left: usize, palette: Palette, separator: &str) -> LinePart {
    if tab_count_to_the_left == 0 {
        return LinePart::default();
    }
    let more_text = if tab_count_to_the_left < 10000 {
        format!(" ← +{} ", tab_count_to_the_left)
    } else {
        " ← +many ".to_string()
    };
    // 238
    // chars length plus separator length on both sides
    let more_text_len = more_text.chars().count() + 2 * separator.chars().count();
    let left_separator = style!(palette.cyan, palette.orange).paint(separator);
    let more_styled_text = style!(palette.black, palette.orange)
        .bold()
        .paint(more_text);
    let right_separator = style!(palette.orange, palette.cyan).paint(separator);
    let more_styled_text = format!(
        "{}",
        ANSIStrings(&[left_separator, more_styled_text, right_separator,])
    );
    LinePart {
        part: more_styled_text,
        len: more_text_len,
    }
}

fn right_more_message(
    tab_count_to_the_right: usize,
    palette: Palette,
    separator: &str,
) -> LinePart {
    if tab_count_to_the_right == 0 {
        return LinePart::default();
    };
    let more_text = if tab_count_to_the_right < 10000 {
        format!(" +{} → ", tab_count_to_the_right)
    } else {
        " +many → ".to_string()
    };
    // chars length plus separator length on both sides
    let more_text_len = more_text.chars().count() + 2 * separator.chars().count();
    let left_separator = style!(palette.cyan, palette.orange).paint(separator);
    let more_styled_text = style!(palette.black, palette.orange)
        .bold()
        .paint(more_text);
    let right_separator = style!(palette.orange, palette.cyan).paint(separator);
    let more_styled_text = format!(
        "{}",
        ANSIStrings(&[left_separator, more_styled_text, right_separator,])
    );
    LinePart {
        part: more_styled_text,
        len: more_text_len,
    }
}

fn tab_line_prefix(session_name: Option<&str>, palette: Palette, cols: usize) -> Vec<LinePart> {
    let prefix_text = " Zellij ".to_string();

    let prefix_text_len = prefix_text.chars().count();
    let prefix_styled_text = style!(palette.white, palette.cyan)
        .bold()
        .paint(prefix_text);
    let mut parts = vec![LinePart {
        part: format!("{}", prefix_styled_text),
        len: prefix_text_len,
    }];
    if let Some(name) = session_name {
        let name_part = format!("({}) ", name);
        let name_part_len = name_part.chars().count();
        let name_part_styled_text = style!(palette.white, palette.cyan).bold().paint(name_part);
        if cols.saturating_sub(prefix_text_len) >= name_part_len {
            parts.push(LinePart {
                part: format!("{}", name_part_styled_text),
                len: name_part_len,
            })
        }
    }
    parts
}

pub fn tab_separator(capabilities: PluginCapabilities) -> &'static str {
    if !capabilities.arrow_fonts {
        ARROW_SEPARATOR
    } else {
        ""
    }
}

pub fn tab_line(
    session_name: Option<&str>,
    mut all_tabs: Vec<LinePart>,
    active_tab_index: usize,
    cols: usize,
    palette: Palette,
    capabilities: PluginCapabilities,
) -> Vec<LinePart> {
    let mut tabs_after_active = all_tabs.split_off(active_tab_index);
    let mut tabs_before_active = all_tabs;
    let active_tab = if !tabs_after_active.is_empty() {
        tabs_after_active.remove(0)
    } else {
        tabs_before_active.pop().unwrap()
    };
    let mut prefix = tab_line_prefix(session_name, palette, cols);
    let prefix_len = get_current_title_len(&prefix);

    // if active tab alone won't fit in cols, don't draw any tabs
    if prefix_len + active_tab.len > cols {
        return prefix;
    }

    let mut tabs_to_render = vec![active_tab];

    populate_tabs_in_tab_line(
        &mut tabs_before_active,
        &mut tabs_after_active,
        &mut tabs_to_render,
        cols.saturating_sub(prefix_len),
        palette,
        capabilities,
    );
    prefix.append(&mut tabs_to_render);
    prefix
}
