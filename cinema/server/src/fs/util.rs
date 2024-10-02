const MAX_LEN: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedVec<T> {
	inner: Vec<Option<T>>,
}

impl<T> IndexedVec<T> {
	pub fn new() -> Self {
		Self { inner: vec![] }
	}

	pub fn with_capacity(cap: usize) -> Self {
		Self {
			inner: Vec::with_capacity(cap),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.inner.iter().all(Option::is_none)
	}

	pub fn get_mut(&mut self, idx: usize) -> &mut Option<T> {
		assert!(idx <= MAX_LEN);

		if self.inner.len() <= idx {
			self.inner.resize_with(idx + 1, || None);
		}

		self.inner.get_mut(idx).unwrap()
	}

	pub fn remove(&mut self, idx: usize) -> Option<T> {
		self.get_mut(idx).take()
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

impl<T> From<Vec<T>> for IndexedVec<T> {
	fn from(v: Vec<T>) -> Self {
		Self {
			inner: v.into_iter().map(Some).collect(),
		}
	}
}

impl<T> FromIterator<Option<T>> for IndexedVec<T> {
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = Option<T>>,
	{
		Self {
			inner: iter.into_iter().collect(),
		}
	}
}

impl<T> FromIterator<(usize, T)> for IndexedVec<T> {
	fn from_iter<I>(iter: I) -> Self
	where
		I: IntoIterator<Item = (usize, T)>,
	{
		let iter = iter.into_iter();
		let hint = iter.size_hint();
		let mut me = Self::with_capacity(hint.1.unwrap_or(hint.0));

		for (idx, t) in iter {
			// todo this could be more efficient
			me.set(idx, t);
		}

		me
	}
}

pub struct IntoIter<T>(std::vec::IntoIter<Option<T>>);

impl<T> IntoIterator for IndexedVec<T> {
	type Item = T;
	type IntoIter = IntoIter<T>;

	fn into_iter(self) -> Self::IntoIter {
		IntoIter(self.inner.into_iter())
	}
}

impl<T> Iterator for IntoIter<T> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(t) = self.0.next()? {
				return Some(t);
			}
		}
	}
}

pub struct Iter<'a, T>(std::slice::Iter<'a, Option<T>>);

impl<'a, T> IntoIterator for &'a IndexedVec<T> {
	type Item = &'a T;
	type IntoIter = Iter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		Iter(self.inner.iter())
	}
}

impl<'a, T> Iterator for Iter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(t) = self.0.next()? {
				return Some(t);
			}
		}
	}
}

pub struct IterMut<'a, T>(std::slice::IterMut<'a, Option<T>>);

impl<'a, T> IntoIterator for &'a mut IndexedVec<T> {
	type Item = &'a mut T;
	type IntoIter = IterMut<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		IterMut(self.inner.iter_mut())
	}
}

impl<'a, T> Iterator for IterMut<'a, T> {
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if let Some(t) = self.0.next()? {
				return Some(t);
			}
		}
	}
}
