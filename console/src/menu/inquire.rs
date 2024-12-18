use chrono::{DateTime, Local};

pub(crate) mod back_menu;
pub(crate) mod do_now_list_menu;
pub(crate) mod select_higher_importance_than_this;
pub(crate) mod update_item_summary;

fn parse_exact_or_relative_datetime(input: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    match duration_str::parse(input) {
        Ok(exact_start) => Some(now + exact_start),
        Err(_) => match dateparser::parse(&input) {
            Ok(exact_start) => Some(exact_start.into()),
            Err(_e) => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::parse_exact_or_relative_datetime;

    #[test]
    fn test_parse_exact_or_relative_datetime_writing_a_complete_datetime_with_a_full_time() {
        let ignored = Local::now();
        assert_eq!(
            parse_exact_or_relative_datetime("1/15/2025 3:00pm", ignored),
            Some(
                Local
                    .with_ymd_and_hms(2025, 1, 15, 15, 0, 0)
                    .earliest()
                    .unwrap()
            )
        );
    }

    #[test]
    fn test_parse_exact_or_relative_datetime_an_invalid_date_returns_none() {
        let ignored = Local::now();
        assert_eq!(
            parse_exact_or_relative_datetime("invalid date", ignored),
            None
        );
    }
}
