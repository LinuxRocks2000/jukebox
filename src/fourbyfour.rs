pub struct FourByFour {
    inputs : gpiod::Lines<gpiod::Input>,
    outputs : gpiod::Lines<gpiod::Output>
}

#[derive(Copy, Clone)]
pub struct FourByFourState {
    state     : u16,
    map       : [u8; 16]
}

pub struct FourByFourD {
    old : FourByFourState, // this wastes a little memory - 32 bytes of it, to be exact.
    new : FourByFourState
}


impl FourByFour {
    pub fn new(rows : [u32; 4], cols : [u32; 4]) -> Self {
        let chip = gpiod::Chip::new("gpiochip0").unwrap();
        let input_opts = gpiod::Options::input(rows).bias(gpiod::Bias::PullDown);
        let output_opts = gpiod::Options::output(cols);
        FourByFour {
            inputs : chip.request_lines(input_opts).unwrap(),
            outputs : chip.request_lines(output_opts).unwrap()
        }
    }

    pub fn read_pad_raw(&mut self) -> u16 {
        let mut ret : u16 = 0;
        for out in 0..4 {
            let mut outputs = [false, false, false, false];
            outputs[out] = true;
            self.outputs.set_values(outputs).unwrap();
            let values = self.inputs.get_values([false; 4]).unwrap();
            for inp in 0..4 {
                if values[inp] {
                    ret += 1 << (out * 4 + inp);
                }
            }
        }
        ret
    }

    pub fn read_pad_mapped(&mut self, map : [u8; 16]) -> FourByFourState {
        FourByFourState {
            state : self.read_pad_raw(),
            map
        }
    }

    pub fn read_pad(&mut self) -> FourByFourState {
        self.read_pad_mapped([
            b'1', b'2', b'3', b'A',

            b'4', b'5', b'6', b'B',

            b'7', b'8', b'9', b'C',

            b'*', b'0', b'#', b'D'])
    }
}


impl FourByFourState {
    pub fn empty() -> Self {
        FourByFourState {
            state : 0,
            map : [0; 16]
        }
    }

    pub fn is_pressed_raw(&self, thing : u16) -> bool {
        self.state & (1 << thing) > 0
    }

    pub fn is_pressed(&self, thing : u8) -> bool {
        for button in 0..16 as u16 {
            if self.map[button as usize] == thing && self.is_pressed_raw(button) {
                return true;
            }
        }
        return false;
    }

    pub fn into_vec(&self) -> Vec<u8> {
        let mut ret : Vec<u8> = vec![];
        for button in 0..16 as u16 {
            if self.is_pressed_raw(button) {
                ret.push(self.map[button as usize]);
            }
        }
        ret
    }

    pub fn aint(&self, other : FourByFourState) -> FourByFourD {
        FourByFourD {
            old : *self,
            new : other
        }
    }
}

impl FourByFourD {
    pub fn released(&self, button : u8) -> bool { // return if the button was released between self.old and self.new
        if !self.old.is_pressed(button) {
            return false;
        }
        if self.new.is_pressed(button) {
            return false;
        }
        return true;
    }
}