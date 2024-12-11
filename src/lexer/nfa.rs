pub struct NfaBuilder<T>(std::marker::PhantomData<T>);

impl<T> NfaBuilder<T> {}

pub struct Nfa<T>(std::marker::PhantomData<T>);

impl<T> Nfa<T> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}
