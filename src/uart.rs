use crossbeam_channel::Sender;
use std::collections::VecDeque;

pub struct UartRx {
    circ_buffer: Vec<u8>,
    pointer: usize,
    byte: u8,
    pos: usize,
    in_frame: bool,
    middle_zeroes: usize,
    middle_samples: usize,
    middle_samples_threshold: usize,
    samples_back_to_start: usize,
    samples_per_symbol: usize,
    to_pty: Sender<u8>,
}

impl UartRx {
    pub fn new(samples_per_symbol: usize, to_pty: Sender<u8>) -> Self {
        let middle_samples = 3 * samples_per_symbol / 16;
        let middle_samples_threshold = 5 * middle_samples / 6;
        let samples_back_to_start = samples_per_symbol / 2 + middle_samples / 2;
        let circ_buffer = vec![1; samples_per_symbol];
        UartRx {
            samples_per_symbol,
            to_pty,
            middle_samples,
            middle_samples_threshold,
            samples_back_to_start,
            circ_buffer,
            in_frame: false,
            byte: 0,
            middle_zeroes: 0,
            pointer: 0,
            pos: 0,
        }
    }

    pub fn put_samples(&mut self, buffer: &[u8]) {
        for sample in buffer {
            self.pointer = (self.pointer + 1) % self.circ_buffer.len();
            self.circ_buffer[self.pointer] = *sample;
            let middle_samples_start = (self.pointer + self.circ_buffer.len() - self.middle_samples) % self.circ_buffer.len();
            self.middle_zeroes += self.circ_buffer[middle_samples_start] as usize - *sample as usize;
            
            if !self.in_frame {
                let frame_start = (self.pointer + self.circ_buffer.len() - self.samples_back_to_start) % self.circ_buffer.len();
                
                if self.middle_zeroes >= self.middle_samples_threshold && self.circ_buffer[frame_start] == 0 {
                    self.in_frame = true;
                    self.pos = self.samples_back_to_start;
                    self.byte = 0;
                }
            } else {
                self.pos += 1;
                if self.pos % self.samples_per_symbol == self.samples_per_symbol / 2 {
                    
                    self.byte = (self.byte >> 1) | (sample << 7);
                    if self.pos / self.samples_per_symbol == 8 {
                        self.to_pty.send(self.byte).unwrap();
                    }
                    if self.pos / self.samples_per_symbol == 9 {
                        self.in_frame = false;
                    }
                }
            }
        }
    }
}

pub struct UartTx {
    samples_per_symbol: usize,
    samples: VecDeque<u8>,
}

impl UartTx {
    pub fn new(samples_per_symbol: usize) -> Self {
        Self {
            samples_per_symbol,
            samples: VecDeque::new(),
        }
    }

    fn put_bit(&mut self, bit: u8) {
        for _ in 0..self.samples_per_symbol {
            self.samples.push_back(bit);
        }
    }

    pub fn put_byte(&mut self, mut byte: u8) {
        self.put_bit(0); // start bit
        for _ in 0..8 {
            self.put_bit(byte & 1);
            byte >>= 1;
        }
        self.put_bit(1); // stop bit
    }

    pub fn get_samples(&mut self, buffer: &mut [u8]) {
        for i in 0..buffer.len() {
            buffer[i] = self.samples.pop_front().unwrap_or(1);
        }
    }
}
