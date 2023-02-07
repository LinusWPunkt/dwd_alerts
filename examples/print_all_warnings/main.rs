use dwd_alerts::{Warning, WarningList};

fn main() {
    let warning_list = WarningList::get_new().unwrap();

    warning_list
        .into_iter()
        .filter(|f| f.is_current())
        .for_each(|w| println!("{w:?}"));
}
