use chrono::{Local, NaiveDateTime, TimeDelta};

use crate::data::{Config, Retention};

pub trait WithinRange {
	fn is_within(&self, config: &Config) -> bool;
}

impl WithinRange for NaiveDateTime {

	fn is_within(&self, config: &Config) -> bool {
	    match config.retention {
	        Retention::Days(days) => {
	        	let cutoff_date = *self + TimeDelta::days(days.into());
	        	cutoff_date > Local::now().naive_local()
	        },
	        Retention::Unknown => panic!("Unset retetion")
	    }
	}

}