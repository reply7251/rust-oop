use std::pin::Pin;

use rust_oop::class;

#[test]
fn test() {

    let car = Car::new(String::from("Car"), String::from("car1"));
    car.print_self();
    let car2 = Car::with(String::from("car2"));
    car2.print_self();
    car2.print_mro();
    let land_vehicle = LandVehicle::with();
    land_vehicle.print_self();
    land_vehicle.print_mro();
}

class!{
    struct Vehicle {
        _type: String
    }
    impl Vehicle {
        fn get_name(&self) -> String { String::from("unnamed") }
        fn get_type(&self) -> String { this._type.clone() }
        fn print_self(&self) {
            println!("Type: {}, Name: {};", self.get_type(), self.get_name());
        }
        fn print_mro(&self) {
            println!("Vehicle");
        }
    }
}
class!{
    extends Vehicle;
    struct LandVehicle { }

    impl LandVehicle {
        #[keep]
        fn with() -> Pin<Box<LandVehicle>> where Self: Sized{
            Self::new(String::from("LandVehicle"))
        }

        fn print_mro(&self) {
            println!("LandVehicle");
            _super.print_mro();
        }
    }
}
class!{
    extends LandVehicle;
    pub struct Car {
        name: String
    }
    
    impl Car {
        fn get_name(&self) -> String{
            this.name.clone()
        }
        #[keep]
        fn with(name: String) -> Pin<Box<Self>> where Self : Sized {
            Self::new(String::from("Car"), name)
        }
        fn print_mro(&self) {
            println!("Car");
            _super.print_mro();
        }
    }
}