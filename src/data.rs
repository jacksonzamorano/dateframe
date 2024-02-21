use std::fs;

pub trait ErrorDisplay {
	fn display(self) -> String;
}

pub struct Config {
    pub format: String,
    pub retention: Retention,
    pub deep: bool,
    pub refresh: u64
}

impl Config {
	fn new() -> Config {
		Config { format: String::new(), retention: Retention::Unknown, deep: true, refresh: 360 }
	}

	fn line_k_v(line: &str) -> Option<(String, String)> {
		let mut split = line.split('=');
       	Some((split.next()?.to_string(), split.next()?.to_string()))
	}

	fn patch(&mut self, key: &str, value: String) -> Result<(), ConfigError> {
		match key {
			"format" => self.format = value,
			"retention" => self.retention = Retention::from_string(&value)?,
			"deep" => self.deep = value == "true",
			"refresh" => self.refresh = value.parse().map_err(|_| ConfigError::InvalidRefresh)?,
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
        if config.format.is_empty() { return Err(ConfigError::NoFormat) }
        if config.retention == Retention::Unknown { return Err(ConfigError::NoRetention) }
        Ok(config)
    }
}

pub enum ConfigError {
    CannotOpenFile,
    NoFormat,
    NoRetention,
    InvalidKey,
    InvalidRetention(RetentionError),
    InvalidRefresh
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
	    }
	}
}

impl From<RetentionError> for ConfigError {
	fn from(x: RetentionError) -> Self { 
		ConfigError::InvalidRetention(x)
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
	        Self::InvalidValue => "Invalid number provided"
	    }.to_string()
	}
}