use std::fs;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

macro_rules! println_info {
	($config:ident, $str: literal, $($tts:tt)*) => {
		if ($config.log.show_info()) {
			println!($str, $($tts)*);
		}
	};
	($config: ident, $str: literal) => {
		if ($config.log.show_info()) {
			println!($str);
		}
	}
}

macro_rules! println_debug {
	($config:ident, $str: literal, $($tts:tt)*) => {
		if ($config.log.show_debug()) {
			println!($str, $($tts)*);
		}
	};
	($config: ident, $str: literal) => {
		if ($config.log.show_debug()) {
			println!($str);
		}
	}
}

macro_rules! println_error {
	($config:ident, $str: literal, $($tts:tt)*) => {
		if ($config.log.show_error()) {
			println!($str, $($tts)*);
		}
	};
	($config: ident, $str: literal) => {
		if ($config.log.show_error()) {
			println!($str);
		}
	}
}

pub trait ErrorDisplay {
    fn display(self) -> String;
}

pub struct Config {
    pub format: Vec<String>,
    pub retention: Retention,
    pub deep: bool,
    pub refresh: u64,
    pub remove: Vec<String>,
    pub log: LogLevel,
    pub split_string: Option<String>,
    pub split_join: String,
    pub split_indicies: Vec<usize>,
    pub date_only_behavior: DateOnlyBehavior,
}

impl Config {
    fn new() -> Config {
        Config {
            format: Vec::new(),
            retention: Retention::Unknown,
            deep: true,
            refresh: 360,
            remove: Vec::new(),
            log: LogLevel::Info,
            split_string: None,
            split_join: String::new(),
            split_indicies: Vec::new(),
            date_only_behavior: DateOnlyBehavior::Start,
        }
    }

    fn line_k_v(line: &str) -> Option<(String, String)> {
        let mut split = line.split('=');
        Some((split.next()?.to_string(), split.next()?.to_string()))
    }

    fn patch(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
        match key {
            "format" => self.format.push(value),
            "retention" => self.retention = Retention::from_string(&value)?,
            "deep" => self.deep = value == "true",
            "refresh" => self.refresh = value.parse().map_err(|_| ConfigError::InvalidRefresh)?,
            "remove" => self.remove.push(value), 
            "log" => self.log = LogLevel::from_string(&value),
            "split_string" => self.split_string = Some(value),
            "split_join" => self.split_join = value,
            "split_index" => self.split_indicies.push(value.parse().map_err(|_| ConfigError::InvalidSpaceSplit)?),
            "date_only_behavior" => self.date_only_behavior = DateOnlyBehavior::from_string(&value)?,
            _ => return Err(ConfigError::InvalidKey),
        }
        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Config, ConfigError> {
        let mut config = Config::new();
        let contents = fs::read(path)
            .ok()
            .and_then(|contents_bytes| String::from_utf8(contents_bytes).ok())
            .ok_or(ConfigError::CannotOpenFile)?;
        for l in contents.lines() {
            if let Some((key, value)) = Config::line_k_v(l) {
                config.patch(&key, value)?;
            }
        }
        if config.format.is_empty() {
            return Err(ConfigError::NoFormat);
        }
        if config.retention == Retention::Unknown {
            return Err(ConfigError::NoRetention);
        }
        Ok(config)
    }

    pub fn format_name(&self, name: &str) -> String {
    	let mut formatted = name.to_string();
        if !self.remove.is_empty() {
            for r in &self.remove {
                formatted = name.replace(r, "");
            }
        }
        if let Some(split) = &self.split_string {
            let splits = formatted.split(split).collect::<Vec<_>>();
            let mut split_contents: Vec<String> = Vec::new();
            for idx in &self.split_indicies {
                if let Some(contents) = splits.get(*idx) {
                    split_contents.push(contents.to_string());
                } else {
                    println_info!(self, "\t\tIgnored index {} because no such index.", idx);
                }
            }
            formatted = split_contents.join(&self.split_join);
        }
        formatted

    }
}

pub enum ConfigError {
    CannotOpenFile,
    NoFormat,
    NoRetention,
    InvalidKey,
    InvalidRetention(RetentionError),
    InvalidRefresh,
    InvalidSpaceSplit,
    InvalidDateOnlyBehavior(DateOnlyBehaviorError),
}

impl ErrorDisplay for ConfigError {
    fn display(self) -> String {
        match self {
            Self::CannotOpenFile => "Cannot open file!".to_string(),
            Self::NoFormat => "No format value".to_string(),
            Self::NoRetention => "No retention value".to_string(),
            Self::InvalidKey => "Invalid key provided in config".to_string(),
            Self::InvalidRetention(ret) => format!("Invalid retetion: {}", ret.display()),
            Self::InvalidRefresh => "Invalid refresh value".to_string(),
            Self::InvalidSpaceSplit => "Invalid space split value".to_string(),
            Self::InvalidDateOnlyBehavior(err) => format!("Invalid date only behavior: {}", err.display())
        }
    }
}

impl From<RetentionError> for ConfigError {
    fn from(x: RetentionError) -> Self {
        ConfigError::InvalidRetention(x)
    }
}

impl From<DateOnlyBehaviorError> for ConfigError {
	fn from(value: DateOnlyBehaviorError) -> Self {
	    ConfigError::InvalidDateOnlyBehavior(value)
	}
}

#[derive(PartialEq)]
pub enum DateOnlyBehavior {
	Start,
	Noon,
	End,
	Hour(u32)
}
impl DateOnlyBehavior {
	pub fn from_string(s: &str) -> Result<DateOnlyBehavior, DateOnlyBehaviorError> {
		if let Some(hour_string) = s.strip_prefix('h') {
			return Ok(DateOnlyBehavior::Hour(hour_string.parse().map_err(|_| DateOnlyBehaviorError::InvalidHour)?));
		}
		Ok(match s {
		    "start" => DateOnlyBehavior::Start,
		    "noon" => DateOnlyBehavior::Noon,
		    "end" => DateOnlyBehavior::End,
		    _ => DateOnlyBehavior::Start
		})
	}
	pub fn add_to_date(&self, d: NaiveDate) -> NaiveDateTime {
		let t = match self {
		    DateOnlyBehavior::Start => NaiveTime::from_hms_opt(0, 0, 0),
		    DateOnlyBehavior::Noon => NaiveTime::from_hms_opt(12, 0, 0),
		    DateOnlyBehavior::End => NaiveTime::from_hms_opt(23, 59, 59),
		    DateOnlyBehavior::Hour(x) => NaiveTime::from_hms_opt(*x, 0, 0),
		}.unwrap();
		d.and_time(t)
	}
}

pub enum DateOnlyBehaviorError {
	InvalidHour
}
impl ErrorDisplay for DateOnlyBehaviorError {
	fn display(self) -> String {
	    match self {
	        DateOnlyBehaviorError::InvalidHour => "Invalid hour provided"
	    }.to_string()
	}
}

#[derive(PartialEq)]
pub enum Retention {
    Unknown,
    Days(i32),
}

impl Retention {
    pub fn from_string(data: &str) -> Result<Retention, RetentionError> {
        match data.chars().last().unwrap_or(' ') {
            'd' => Ok(Retention::Days(
                data[0..data.chars().count() - 1]
                    .parse()
                    .map_err(|_| RetentionError::InvalidValue)?,
            )),
            _ => Err(RetentionError::InvalidMeasurement),
        }
    }
}

pub enum RetentionError {
    InvalidMeasurement,
    InvalidValue,
}

impl ErrorDisplay for RetentionError {
    fn display(self) -> String {
        match self {
            Self::InvalidMeasurement => "Invalid measurement provided",
            Self::InvalidValue => "Invalid number provided",
        }
        .to_string()
    }
}


#[derive(PartialEq)]
pub enum LogLevel {
	Debug,
	Info,
	Error,
	Silent
}
impl LogLevel {
	pub fn from_string(v: &str) -> LogLevel {
		match v {
			"debug" => LogLevel::Debug,
			"info" => LogLevel::Info,
			"error" => LogLevel::Error,
			"silent" => LogLevel::Silent,
			_ => LogLevel::Info
		}
	}

	pub fn show_info(&self) -> bool {
		*self == LogLevel::Info || *self == LogLevel::Debug
	}

	pub fn show_error(&self) -> bool {
		*self == LogLevel::Error || *self == LogLevel::Info || *self == LogLevel::Debug
	}

	pub fn show_debug(&self) -> bool { *self == LogLevel::Debug }
}
