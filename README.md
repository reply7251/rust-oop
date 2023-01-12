Macro to implement inherit

Provide <code>class</code> to cover a struct itself, it's impl and trait implements that need to inherit.
To borrow as mut, see <code>def_as_mut</code>

macro <code>class</code>

need to wrap struct and implements.

using <code>#\[keep\]</code> for method make that method keep in the original impl.

methods without <code>#\[keep\]</code> will be put into trait <code>\_\_XXX\_\_</code>.

and macro will auto generate a <code>new</code> function which return <code>Pin<Box<Self>></code>.

expression in the method will be converted.

instead of use <code>self</code>, using <code>this</code>.

<code>self</code> will be convert to use <code>unsafe { self.\_\_real\_\_.as_ref().unwrap() }</code>.

<code>self_mut</code> will be convert to use <code>unsafe { self.\_\_real\_\_.as_mut().unwrap() }</code>.

<code>\_super</code> will be convert to use <code>self.\_\_prototype\_\_</code>.

<code>\_super_mut</code> will be convert to use <code>unsafe { self.\_\_prototype\_\_.as_mut().get_unchecked_mut() }</code>.

macro <code>def_as_mut</code>

this macro will define macro <code>as_mut</code>

example:
```rust
class! {
    struct Example {
        data: String    
    }
    impl Example {
        #[keep]
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
    impl Sub { }
}
```
the struct will become:
```rust
struct Example {
    __real__: *mut dyn __Example__,
    _pinned: ::std::marker::PhantomPinned,
    data: String
}
struct Sub {
    __prototype__: ::std::pin::Pin<Box<Example>>
    __real__: *mut dyn __Sub__,
    _pinned: ::std::marker::PhantomPinned,
}
```
to borrow as mut:
```rust
def_as_mut!();

fn main() {
    let mut example = Sub::new("data".to_string());
    as_mut!(example).set_data("modified".to_string());
    assert_eq!(example.get_data(), "modified".to_string());
}
```