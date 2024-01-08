// reads data from an EC11 rotary encoder.

pub struct RotaryEncoder {
    pressed  : bool,
    was_down : bool,
    buffer   : u8,
    position : i32,
    inputs   : gpiod::Lines<gpiod::Input>
}

impl RotaryEncoder {
    pub fn new(button : u32, a : u32, b : u32) -> Self { // a = MSB, b = LSB
        let chip = gpiod::Chip::new("gpiochip0").unwrap();
        let options = gpiod::Options::input([button, a, b]).bias(gpiod::Bias::PullDown);
        RotaryEncoder {
            pressed  : false,
            position : 0,
            was_down : false,
            buffer   : 0,
            inputs   : chip.request_lines(options).unwrap()
        }
    }

    pub fn poll(&mut self) {
        // one direction is 3-1-0-2; 210
        // the other direction is 3-2-0-1; 225
        let values = self.inputs.get_values([false; 3]).unwrap();
        if values[0] {
            self.was_down = true;
        }
        else if self.was_down {
            self.was_down = false;
            self.pressed = true;
        }
        let val : u8 = if values[1] { 2 } else { 0 } + if values[2] { 1 } else { 0 };
        if self.buffer & 3 != val {
            self.buffer = self.buffer << 1;
            self.buffer += if values[1] { 1 } else { 0 };
            self.buffer = self.buffer << 1;
            self.buffer += if values[2] { 1 } else { 0 };
            if self.buffer == 210 {
                self.position += 1;
            }
            else if self.buffer == 225 {
                self.position -= 1;
            }
        }
    }

    pub fn was_pressed(&mut self) -> bool {
        if self.pressed {
            self.pressed = false;
            true
        }
        else {
            false
        }
    }

    pub fn map(&self, min_in : i32, max_in : i32, min_out : f32, max_out : f32) -> f32 {
        let a = if self.position < min_in { min_in } else if self.position > max_in { max_in } else { self.position } - min_in;
        let b = a as f32 / (max_in - min_in) as f32;
        let c = b * (max_out - min_out);
        c + min_out
    }
}