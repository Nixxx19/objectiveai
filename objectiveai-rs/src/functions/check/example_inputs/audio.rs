use rand::Rng;

use crate::agent::completions::message::{InputAudio, RichContentPart};
use crate::functions::expression::{AudioInputSchema, InputValue};

pub const fn permutations(_schema: &AudioInputSchema) -> usize {
    1usize
}

pub fn generate<R: Rng>(_schema: &AudioInputSchema, rng: R) -> Generator<R> {
    Generator { rng }
}

pub struct Generator<R: Rng> {
    rng: R,
}

impl<R: Rng> Iterator for Generator<R> {
    type Item = InputValue;
    fn next(&mut self) -> Option<InputValue> {
        Some(InputValue::RichContentPart(RichContentPart::InputAudio {
            input_audio: InputAudio {
                data: super::string::random_string(&mut self.rng),
                format: super::string::random_string(&mut self.rng),
            },
        }))
    }
}
