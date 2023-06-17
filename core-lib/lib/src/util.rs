use crate::{Error, ErrorKind};

use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;


pub fn mpsc_send<T>(s: &mpsc::Sender<T>, msg: T) -> Result<(), Error> {
	match s.try_send(msg) {
		Ok(()) => Ok(()),
		Err(TrySendError::Closed(_)) => {
			Err(Error::new(ErrorKind::Refused, "mpsc channel closed"))
		},
		Err(TrySendError::Full(_)) => {
			Err(Error::new(
				ErrorKind::TooManyRequests,
				"mpsc channel full"
			))
		}
	}
}