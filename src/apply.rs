pub trait Apply: Sized {
    // Apply a style function to this builder
    fn apply(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

