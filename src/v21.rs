use std::{f32::consts::PI, ops::Rem};

pub struct V21RX {
    // TODO: coloque outros atributos que você precisar aqui
    sampling_period: f32,
    samples_per_symbol: usize,
    omega_mark: f32,
    omega_space: f32,
    circ_buffer: Vec<f32>,
    pointer: usize,
    v0i: f32,
    v0r: f32,
    v1i: f32,
    v1r: f32,
    prev_filtered_decision: f32,
}

impl V21RX {
    pub fn new(
        sampling_period: f32,
        samples_per_symbol: usize,
        omega_mark: f32,
        omega_space: f32,
    ) -> Self {
        // TODO: inicialize seus novos atributos abaixo
        let circ_buffer = vec![0.; samples_per_symbol * 2];
        Self {
            sampling_period,
            samples_per_symbol,
            omega_mark,
            omega_space,
            circ_buffer,
            pointer: 0,
            v0i: 0.,
            v0r: 0.,
            v1i: 0.,
            v1r: 0.,
            prev_filtered_decision: 0.,
        }
    }

    pub fn demodulate(&mut self, in_samples: &[f32], out_samples: &mut [u8]) {
        let r: f32 = 0.9999;
        let l = self.samples_per_symbol as f32;
        let t = self.sampling_period as f32;
        let omega0 = self.omega_space;
        let omega1 = self.omega_mark;
        let alpha = 2.*PI*t*300. / (2.*PI*t*300. + 1.);

        for i in 0..in_samples.len() {
            self.pointer = (self.pointer + 1) % self.circ_buffer.len();
            self.circ_buffer[self.pointer] = in_samples[i];
            let last_symbol = (self.pointer + self.circ_buffer.len() - self.samples_per_symbol) % self.circ_buffer.len();
            
            let v0r = in_samples[i] - r.powf(l) * (omega0*l*t).cos()*self.circ_buffer[last_symbol] + r*(omega0*t).cos()*self.v0r - r*(omega0*t).sin()*self.v0i;
            let v0i = -r.powf(l)*(omega0*l*t).sin()*self.circ_buffer[last_symbol] + r*(omega0*t).cos()*self.v0i + r*(omega0*t).sin()*self.v0r;
            let v1r = in_samples[i] - r.powf(l) * (omega1*l*t).cos()*self.circ_buffer[last_symbol] + r*(omega1*t).cos()*self.v1r - r*(omega1*t).sin()*self.v1i;
            let v1i = -r.powf(l)*(omega1*l*t).sin()*self.circ_buffer[last_symbol] + r*(omega1*t).cos()*self.v1i + r*(omega1*t).sin()*self.v1r;
            
            let decision = v1r*v1r+v1i*v1i -v0r*v0r-v0i*v0i;
            let filtered_decision = self.prev_filtered_decision + alpha * (decision - self.prev_filtered_decision);
            
            self.prev_filtered_decision = filtered_decision;
            self.v0r = v0r;
            self.v0i = v0i;
            self.v1r = v1r;
            self.v1i = v1i;
            
            out_samples[i] = if filtered_decision > 0. || filtered_decision.abs() < 120. {1} else {0};
        }
    }
}

pub struct V21TX {
    sampling_period: f32,
    omega_mark: f32,
    omega_space: f32,
    phase: f32,
}

impl V21TX {
    pub fn new(sampling_period: f32, omega_mark: f32, omega_space: f32) -> Self {
        Self {
            sampling_period,
            omega_mark,
            omega_space,
            phase: 0.,
        }
    }

    pub fn modulate(&mut self, in_samples: &[u8], out_samples: &mut [f32]) {
        debug_assert!(in_samples.len() == out_samples.len());

        for i in 0..in_samples.len() {
            out_samples[i] = self.phase.sin();

            let omega = if in_samples[i] == 0 {
                self.omega_space
            } else {
                self.omega_mark
            };
            self.phase = (self.phase + self.sampling_period * omega).rem(2. * PI);
        }
    }
}
