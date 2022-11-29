macro_rules! impl_auto_incremental {
	($($type:ty),+) => {
		$(
			impl AutoIncremental for $type {
				fn increment(&self) -> Self {
					let mut val = self.clone();
					val.saturating_inc();
					val
				}

				fn initial_value() -> Self {
					0
				}
			}
		)+
	};
}
pub(crate) use impl_auto_incremental;
