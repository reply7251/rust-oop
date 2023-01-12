
use std::pin::Pin;

use rust_oop::{class, def_as_mut};

def_as_mut!();

#[test]
fn main_test() {
    let mut shape1 = Rectangle::new(1.0, 2.0);
    assert_eq!(&shape1.name(), "Rectangle");
    assert_eq!(shape1.cal_size(), 2.0);
    as_mut!(shape1).set_width(3.0);
    assert_eq!(shape1.cal_size(), 6.0);
    
    let mut shape2 = Square::with(3.0);

    assert_eq!(&shape2.name(), "Square");
    assert_eq!(shape2.cal_size(), 9.0);
    as_mut!(shape2).set_length(2.0);
    assert_eq!(shape2.cal_size(), 4.0);
}


trait WithSize {
    fn cal_size(&self) -> f32;
}

class!{
    struct Shape {}
    impl Shape {
        fn name(&self) -> String {
            unimplemented!();
        }
    }
    impl WithSize for Shape {
        fn cal_size(&self) -> f32 {
            unimplemented!();
        }
    }
}

class!{
    extends Shape;
    struct Rectangle {
        width: f32,
        height: f32
    }
    impl Rectangle {
        fn name(&self) -> String {
            String::from("Rectangle")
        }
        fn set_width(&mut self, width: f32) where Self: Sized {
            this.width = width;
        }
        fn set_height(&mut self, height: f32) where Self: Sized {
            this.height = height;
        }
    }
    impl WithSize for Rectangle {
        fn cal_size(&self) -> f32 {
            this.width * this.height
        }
    }
}

class!{
    extends Rectangle;
    struct Square { }
    impl Square {
        fn name(&self) -> String {
            String::from("Square")
        }

        fn with(len: f32) -> Pin<Box<Self>> where Self: Sized {
            Self::new(len, len)
        }

        #[allow(unused_variables)]
        fn set_width(&mut self, width: f32) where Self: Sized {
            panic!("don't use set_width on square.");
        }
        #[allow(unused_variables)]
        fn set_height(&mut self, height: f32) where Self: Sized {
            panic!("don't use set_height on square.");
        }

        fn set_length(&mut self, len: f32) where Self: Sized {
            _super_mut.set_width(len);
            _super_mut.set_height(len);
        }
    }
}
