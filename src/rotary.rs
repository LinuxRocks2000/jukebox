// reads data from an EC11 rotary encoder.

pub struct RotaryEncoder {
    pressed  : bool,
    position : u32,
    inputs   : gpiod::Lines<gpiod::Input>
}

impl RotaryEncoder {
    pub fn new(button : u32, a : u32, b : u32) {
        let chip = gpiod::Chip::new("gpiochip0").unwrap();
        let options = gpiod::Options::input([button, a, b]).bias(gpiod::Bias::PullDown);
        RotaryEncoder {
            pressed  : false,
            position : 0,
            inputs   : chip.request_lines(options).unwrap()
        }
    }
}