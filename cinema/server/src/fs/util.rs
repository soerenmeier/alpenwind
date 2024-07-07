const MAX_LEN: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedVec<T> {
	inner: Vec<Option<T>>,
}

impl<T: Clone> IndexedVec<T> {
	pub fn new() -> Self {
		Self { inner: vec![] }
	}

	pub fn get_mut(&mut self, idx: usize) -> &mut Option<T> {
		assert!(idx <= MAX_LEN);

		if self.inner.len() <= idx {
			self.inner.resize(idx + 1, None);
		}

		self.inner.get_mut(idx).unwrap()
	}

	pub fn set(&mut self, idx: usize, val: T) -> Option<T> {
		self.get_mut(idx).replace(val)
	}

	/// Converts everything until a item is missing
	pub fn into_contiguous_map<F, O>(self, f: F) -> Vec<O>
	where
		F: Fn(T) -> O,
	{
		let mut ve = Vec::with_capacity(self.inner.len());
		for v in self.inner {
			if let Some(v) = v {
				ve.push(f(v));
			} else {
				return ve;
			}
		}

		ve
	}
}
