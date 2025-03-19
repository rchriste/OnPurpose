use std::time::Duration;

use chrono::{DateTime, Utc};

use crate::{
    data_storage::surrealdb_layer::surreal_item::SurrealScheduled, node::item_status::ItemStatus,
};

#[derive(Clone)]
pub struct ScheduledItem<'s> {
    item: &'s ItemStatus<'s>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

pub(crate) trait Scheduled {
    fn contains(&self, item: &ItemStatus) -> bool;
    fn find_next_available_time(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> DateTime<Utc>;
    fn calculate_gap_penalty(&self) -> f64;
    fn calculate_big_to_little_count(&self) -> u32;
}

impl Scheduled for Vec<ScheduledItem<'_>> {
    fn contains(&self, item: &ItemStatus) -> bool {
        self.iter().any(|x| x.item == item)
    }

    fn find_next_available_time(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> DateTime<Utc> {
        //Does anything conflict with the given time? If so then advance the time to the end of that conflict and check again (call this function again). Otherwise just return that time as the starting time.
        let conflict = self.iter().find(|x| {
            //Is there any overlap between the range of x and the range of the passed in start and end?
            (start >= x.start || end >= x.start) && (start <= x.end || end <= x.end)
        });
        match conflict {
            Some(conflict) => {
                let one_minute = Duration::new(60, 0);
                let next_proposed_start = conflict.end + one_minute;
                let amount_added = next_proposed_start - start;
                let next_proposed_end = end + amount_added;
                self.find_next_available_time(next_proposed_start, next_proposed_end)
            }
            None => start,
        }
    }

    fn calculate_gap_penalty(&self) -> f64 {
        let mut gap_penalty_sum = 0.0;
        //This assumes that self is sorted and never overlaps
        for i in 0..self.len() - 1 {
            //Log10 is meant to make shorter gaps have a worse penalty than longer gaps.
            //Also note that elsewhere in the code something is scheduled a minute later than the last
            //thing scheduled and log10(1) is 0 so there is no penalty for a one minute gap.
            let gap_penalty_raw = (self[i + 1].start - self[i].end).num_minutes() as f64;
            if gap_penalty_raw < 1.0 {
                //This is because log10 of something less than 1 is a negative number. It is possible that we would want a larger penalty or maybe this would be better to just be an assert as this scenario shouldn't really happen.
                //gap_penalty_raw = 1.0;
                panic!(
                    "This should never happen, it means that we have things that overlap, or we are not sorted properly, or we have a gap of less than a minute. gap_penalty_raw={}, self[i + 1].start={}, self[i].end={}",
                    gap_penalty_raw,
                    self[i + 1].start,
                    self[i].end
                );
            }
            gap_penalty_sum += gap_penalty_raw.log10();
        }
        gap_penalty_sum
    }

    fn calculate_big_to_little_count(&self) -> u32 {
        let mut big_to_little_count = 0;
        //This assumes that self is sorted, otherwise the calculation is meaningless
        for i in 0..self.len() - 1 {
            if (self[i].end - self[i].start) > (self[i + 1].end - self[i + 1].start) {
                big_to_little_count += 1;
            }
        }
        big_to_little_count
    }
}

impl<'s> ScheduledItem<'s> {
    pub(crate) fn new(item: &'s ItemStatus<'s>, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        ScheduledItem { item, start, end }
    }

    pub(crate) fn get_scheduled_start(&self) -> &DateTime<Utc> {
        &self.start
    }

    pub(crate) fn get_summary(&self) -> &str {
        self.item.get_summary()
    }

    pub(crate) fn get_scheduled_now(&self) -> Option<&'s SurrealScheduled> {
        self.item.get_scheduled_now()
    }

    pub(crate) fn get_now(&self) -> &DateTime<Utc> {
        self.item.get_now()
    }
}
