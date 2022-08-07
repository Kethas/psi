pub type Boxs<T> = Box<[T]>;

pub trait IntoBoxs<T> {
    fn into_boxed_slice(self) -> Boxs<T>;
}

impl<I, T> IntoBoxs<T> for I
where
    I: IntoIterator<Item = T>,
{
    fn into_boxed_slice(self) -> Boxs<T> {
        Box::from_iter(self.into_iter())
    }
}

pub trait Void: Sized {
    fn void(self) -> () {
        ()
    }
}

impl<T> Void for T {}

pub trait RefVoid {
    fn void(&self) -> () {
        ()
    }
}

impl<T> RefVoid for T {}
