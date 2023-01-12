use oop::{class, def_as_mut};
use std::pin::Pin;

def_as_mut!();

#[test]
fn test() {
    let mut example = Sub::new("data".to_string());
    as_mut!(example).set_data("modified".to_string());
    assert_eq!(example.get_data(), "modified".to_string());
}

trait Something {
    
}

class! {
    struct Example {
        data: String    
    }
    impl Example {
        #[keep]
        #[allow(unused)]
        fn with() -> Pin<Box<Self>> where Self: Sized {
            Self::new("example".to_string())
        }
        
        fn set_data(&mut self, data: String) {
            this.data = data;
        }

        fn get_data(&self) -> String {
            this.data.clone()
        }
    }
    impl Something for Example {
        //do something
    }
}

class! {
    extends Example;
    struct Sub { }
    impl Sub {}
}