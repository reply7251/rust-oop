
use std::pin::Pin;

#[test]
fn main_test() {
    let mut test = Test2::new("A".to_string());
    unsafe {Pin::get_unchecked_mut(test.as_mut())}.set_data("B".to_string());
}

struct Test{
    data: String,
    _pinned: std::marker::PhantomPinned,
}

trait RealTest {
    fn with() -> Pin<Box<Self>> where Self : Sized;
    fn set_data(&mut self, data: String);
}

impl Test {
    fn new(data: String) -> Pin<Box<Self>> {
        Box::pin(Self {data: data, _pinned: std::marker::PhantomPinned,})
    }
}

impl RealTest for Test {
    fn with() -> Pin<Box<Self>> where Self : Sized {
        Self::new(String::from("default value"))
    }

    fn set_data(&mut self, data: String) {
        self.data = data;
    }
}

struct Test2 {
    prototype: Pin<Box<Test>>,
    _pinned: std::marker::PhantomPinned,
}

impl RealTest for Test2 {
    fn with() -> Pin<Box<Self>> where Self : Sized {
        Self::new(String::from("default value"))
    }
    fn set_data(&mut self, data: String) {
        unsafe {Pin::get_unchecked_mut( self.prototype.as_mut())}.set_data(data);
    }
}

impl Test2 {
    fn new(data: String) -> Pin<Box<Self>> {
        let prototype = Test::new(data);
        Box::pin(Self {prototype, _pinned: std::marker::PhantomPinned})
    }
}
