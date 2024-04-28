use crate::ffi;

use std::time::{Duration, SystemTime};

use serde::de::{Deserializer, Error};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeout {
	inner: SystemTime,
}

impl Timeout {
	pub fn new(dur: Duration) -> Self {
		Self {
			inner: SystemTime::now() + dur,
		}
	}

	pub fn has_elapsed(&self) -> bool {
		SystemTime::now() > self.inner
	}

	/// returns the time from UNIX_EPOCH
	pub fn as_secs(&self) -> u64 {
		self.inner
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Welcome to the past!")
			.as_secs()
	}

	pub fn from_secs(s: u64) -> Option<Self> {
		SystemTime::UNIX_EPOCH
			.checked_add(Duration::from_secs(s))
			.map(|c| Timeout { inner: c })
	}

	pub fn from_c(s: ffi::c_systemtime) -> Self {
		Self {
			inner: s.to_systemtime(),
		}
	}

	pub fn into_c(self) -> ffi::c_systemtime {
		ffi::c_systemtime::from_systemtime(self.inner)
	}
}

impl Serialize for Timeout {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_u64(self.as_secs())
	}
}

impl<'de> Deserialize<'de> for Timeout {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let num: u64 = Deserialize::deserialize(deserializer)?;
		Self::from_secs(num).ok_or(D::Error::custom("timeout to big"))
	}
}
